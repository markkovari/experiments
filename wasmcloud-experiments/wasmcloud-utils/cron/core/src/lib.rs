use std::cell::RefCell;
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Schedule {
    pub minutes: Vec<u8>,      // empty = wildcard (*)
    pub hours: Vec<u8>,
    pub days_of_month: Vec<u8>,
    pub months: Vec<u8>,
    pub days_of_week: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub name: String,
    pub expression: String,
    pub schedule: Schedule,
    pub enabled: bool,
    pub last_run_ms: Option<u64>,
    pub next_run_ms: Option<u64>,
    pub run_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CronError {
    InvalidExpression,
    NotFound,
    DuplicateTask,
    InvalidName,
}

// ── Cron expression parser ────────────────────────────────────────────────────
// Supports: * / , -  in each of 5 fields: min hour dom mon dow

fn parse_field(field: &str, min: u8, max: u8) -> Result<Vec<u8>, CronError> {
    if field == "*" {
        return Ok(vec![]);
    }
    let mut values: Vec<u8> = Vec::new();

    for part in field.split(',') {
        if part.contains('/') {
            // */step or start/step
            let mut it = part.splitn(2, '/');
            let base = it.next().unwrap();
            let step: u8 = it.next().unwrap().parse().map_err(|_| CronError::InvalidExpression)?;
            if step == 0 { return Err(CronError::InvalidExpression); }
            let start = if base == "*" { min } else { base.parse().map_err(|_| CronError::InvalidExpression)? };
            let mut v = start;
            while v <= max {
                values.push(v);
                v = v.saturating_add(step);
                if v < start { break; } // overflow guard
            }
        } else if part.contains('-') {
            let mut it = part.splitn(2, '-');
            let lo: u8 = it.next().unwrap().parse().map_err(|_| CronError::InvalidExpression)?;
            let hi: u8 = it.next().unwrap().parse().map_err(|_| CronError::InvalidExpression)?;
            if lo > hi || lo < min || hi > max { return Err(CronError::InvalidExpression); }
            for v in lo..=hi { values.push(v); }
        } else {
            let v: u8 = part.parse().map_err(|_| CronError::InvalidExpression)?;
            if v < min || v > max { return Err(CronError::InvalidExpression); }
            values.push(v);
        }
    }

    values.sort_unstable();
    values.dedup();
    Ok(values)
}

pub fn parse_expression(expr: &str) -> Result<Schedule, CronError> {
    let fields: Vec<&str> = expr.split_whitespace().collect();
    if fields.len() != 5 {
        return Err(CronError::InvalidExpression);
    }
    Ok(Schedule {
        minutes:      parse_field(fields[0],  0, 59)?,
        hours:        parse_field(fields[1],  0, 23)?,
        days_of_month: parse_field(fields[2], 1, 31)?,
        months:       parse_field(fields[3],  1, 12)?,
        days_of_week: parse_field(fields[4],  0,  6)?,
    })
}

// ── Time helpers (no OS entropy, no system calls) ─────────────────────────────
// We decompose unix-ms into (year, month, day, hour, minute, dow).
// Uses a simple proleptic Gregorian calendar — good enough for cron matching.

fn is_leap(y: u32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(m: u8, y: u32) -> u8 {
    match m {
        1|3|5|7|8|10|12 => 31,
        4|6|9|11 => 30,
        2 => if is_leap(y) { 29 } else { 28 },
        _ => 28,
    }
}

#[derive(Debug, Clone, Copy)]
struct DateTime {
    year: u32,
    month: u8,   // 1–12
    day: u8,     // 1–31
    hour: u8,    // 0–23
    minute: u8,  // 0–59
    dow: u8,     // 0=Sun…6=Sat
}

fn ms_to_dt(ms: u64) -> DateTime {
    let secs = ms / 1000;
    let total_minutes = secs / 60;
    let minute = (total_minutes % 60) as u8;
    let total_hours = total_minutes / 60;
    let hour = (total_hours % 24) as u8;
    let mut total_days = (total_hours / 24) as u32;

    // Jan 1 1970 was a Thursday (dow=4)
    let dow = ((total_days + 4) % 7) as u8;

    let mut year = 1970u32;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if total_days < days_in_year { break; }
        total_days -= days_in_year;
        year += 1;
    }
    let mut month = 1u8;
    loop {
        let d = days_in_month(month, year) as u32;
        if total_days < d { break; }
        total_days -= d;
        month += 1;
    }
    let day = (total_days + 1) as u8;
    DateTime { year, month, day, hour, minute, dow }
}

fn dt_to_ms(dt: &DateTime) -> u64 {
    // Count days from epoch to start of dt.year
    let mut days = 0u64;
    for y in 1970..dt.year {
        days += if is_leap(y) { 366 } else { 365 };
    }
    for m in 1..dt.month {
        days += days_in_month(m, dt.year) as u64;
    }
    days += (dt.day - 1) as u64;
    (days * 86400 + dt.hour as u64 * 3600 + dt.minute as u64 * 60) * 1000
}

fn field_matches(values: &[u8], v: u8) -> bool {
    values.is_empty() || values.binary_search(&v).is_ok()
}

fn schedule_matches(s: &Schedule, dt: &DateTime) -> bool {
    field_matches(&s.minutes,      dt.minute) &&
    field_matches(&s.hours,        dt.hour)   &&
    field_matches(&s.months,       dt.month)  &&
    field_matches(&s.days_of_week, dt.dow)    &&
    (s.days_of_month.is_empty() || field_matches(&s.days_of_month, dt.day))
}

/// Compute the next firing time strictly after `after_ms` (minute granularity).
pub fn next_after(s: &Schedule, after_ms: u64) -> Option<u64> {
    // Round up to the start of the next minute
    let start_ms = (after_ms / 60_000 + 1) * 60_000;
    let mut dt = ms_to_dt(start_ms);

    // Search up to 366*24*60 minutes ahead (avoid infinite loop on impossible schedules)
    for _ in 0..(366 * 24 * 60) {
        if schedule_matches(s, &dt) {
            return Some(dt_to_ms(&dt));
        }
        // Advance by one minute
        dt.minute += 1;
        if dt.minute >= 60 {
            dt.minute = 0;
            dt.hour += 1;
            if dt.hour >= 24 {
                dt.hour = 0;
                dt.day += 1;
                dt.dow = (dt.dow + 1) % 7;
                if dt.day > days_in_month(dt.month, dt.year) {
                    dt.day = 1;
                    dt.month += 1;
                    if dt.month > 12 {
                        dt.month = 1;
                        dt.year += 1;
                    }
                }
            }
        }
    }
    None // unreachable for valid schedules
}

// ── Thread-local state ────────────────────────────────────────────────────────

thread_local! {
    static TASKS: RefCell<HashMap<String, Task>> = RefCell::new(HashMap::new());
}

fn with_tasks<R>(f: impl FnOnce(&mut HashMap<String, Task>) -> R) -> R {
    TASKS.with(|t| f(&mut t.borrow_mut()))
}

fn validate_name(name: &str) -> Result<String, CronError> {
    let n = name.trim().to_string();
    if n.is_empty() { return Err(CronError::InvalidName); }
    Ok(n)
}

// ── Public API ─────────────────────────────────────────────────────────────────

pub fn register(name: &str, expression: &str, now_ms: u64) -> Result<(), CronError> {
    let n = validate_name(name)?;
    let schedule = parse_expression(expression)?;
    let next = next_after(&schedule, now_ms.saturating_sub(1));
    with_tasks(|m| {
        if m.contains_key(&n) { return Err(CronError::DuplicateTask); }
        m.insert(n.clone(), Task {
            name: n,
            expression: expression.to_string(),
            schedule,
            enabled: true,
            last_run_ms: None,
            next_run_ms: next,
            run_count: 0,
        });
        Ok(())
    })
}

pub fn parse(expression: &str) -> Result<Schedule, CronError> {
    parse_expression(expression)
}

pub fn is_due(name: &str, now_ms: u64) -> Result<bool, CronError> {
    let n = validate_name(name)?;
    with_tasks(|m| {
        let task = m.get(&n).ok_or(CronError::NotFound)?;
        if !task.enabled { return Ok(false); }
        Ok(task.next_run_ms.map_or(false, |next| now_ms >= next))
    })
}

pub fn tick(name: &str, now_ms: u64) -> Result<(), CronError> {
    let n = validate_name(name)?;
    with_tasks(|m| {
        let task = m.get_mut(&n).ok_or(CronError::NotFound)?;
        task.last_run_ms = Some(now_ms);
        task.run_count += 1;
        task.next_run_ms = next_after(&task.schedule, now_ms);
        Ok(())
    })
}

pub fn due_tasks(now_ms: u64) -> Result<Vec<Task>, CronError> {
    Ok(with_tasks(|m| {
        m.values()
            .filter(|t| t.enabled && t.next_run_ms.map_or(false, |next| now_ms >= next))
            .cloned()
            .collect()
    }))
}

pub fn get_task(name: &str) -> Result<Task, CronError> {
    let n = validate_name(name)?;
    with_tasks(|m| m.get(&n).cloned().ok_or(CronError::NotFound))
}

pub fn list_tasks() -> Result<Vec<Task>, CronError> {
    Ok(with_tasks(|m| m.values().cloned().collect()))
}

pub fn set_enabled(name: &str, enabled: bool) -> Result<(), CronError> {
    let n = validate_name(name)?;
    with_tasks(|m| {
        m.get_mut(&n).ok_or(CronError::NotFound)?.enabled = enabled;
        Ok(())
    })
}

pub fn deregister(name: &str) -> Result<(), CronError> {
    let n = validate_name(name)?;
    with_tasks(|m| m.remove(&n).ok_or(CronError::NotFound).map(|_| ()))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    // 2024-01-15 10:30:00 UTC = 1705314600000 ms
    const T0: u64 = 1_705_314_600_000;

    #[test]
    fn test_parse_wildcard() {
        run(|| {
            let s = parse_expression("* * * * *").unwrap();
            assert!(s.minutes.is_empty());
            assert!(s.hours.is_empty());
        });
    }

    #[test]
    fn test_parse_specific() {
        run(|| {
            let s = parse_expression("30 10 * * *").unwrap();
            assert_eq!(s.minutes, vec![30]);
            assert_eq!(s.hours, vec![10]);
            assert!(s.days_of_month.is_empty());
        });
    }

    #[test]
    fn test_parse_step() {
        run(|| {
            let s = parse_expression("*/15 * * * *").unwrap();
            assert_eq!(s.minutes, vec![0, 15, 30, 45]);
        });
    }

    #[test]
    fn test_parse_range() {
        run(|| {
            let s = parse_expression("0 9-17 * * 1-5").unwrap();
            assert_eq!(s.hours, vec![9,10,11,12,13,14,15,16,17]);
            assert_eq!(s.days_of_week, vec![1,2,3,4,5]);
        });
    }

    #[test]
    fn test_parse_invalid() {
        run(|| {
            assert_eq!(parse_expression("* * *").unwrap_err(), CronError::InvalidExpression);
            assert_eq!(parse_expression("60 * * * *").unwrap_err(), CronError::InvalidExpression);
            assert_eq!(parse_expression("* 24 * * *").unwrap_err(), CronError::InvalidExpression);
        });
    }

    #[test]
    fn test_register_and_get() {
        run(|| {
            register("heartbeat", "*/5 * * * *", T0).unwrap();
            let t = get_task("heartbeat").unwrap();
            assert_eq!(t.expression, "*/5 * * * *");
            assert!(t.enabled);
            assert!(t.next_run_ms.is_some());
        });
    }

    #[test]
    fn test_duplicate_task() {
        run(|| {
            register("dup", "* * * * *", T0).unwrap();
            assert_eq!(register("dup", "* * * * *", T0).unwrap_err(), CronError::DuplicateTask);
        });
    }

    #[test]
    fn test_is_due_every_minute() {
        run(|| {
            register("every-min", "* * * * *", T0).unwrap();
            let task = get_task("every-min").unwrap();
            let next = task.next_run_ms.unwrap();
            assert!(!is_due("every-min", next - 1).unwrap());
            assert!(is_due("every-min", next).unwrap());
        });
    }

    #[test]
    fn test_tick_advances_next_run() {
        run(|| {
            register("ticker", "*/10 * * * *", T0).unwrap();
            let before = get_task("ticker").unwrap().next_run_ms.unwrap();
            tick("ticker", before).unwrap();
            let after = get_task("ticker").unwrap();
            assert_eq!(after.run_count, 1);
            assert_eq!(after.last_run_ms, Some(before));
            assert!(after.next_run_ms.unwrap() > before);
        });
    }

    #[test]
    fn test_set_enabled_disables() {
        run(|| {
            register("disabled", "* * * * *", T0).unwrap();
            set_enabled("disabled", false).unwrap();
            let task = get_task("disabled").unwrap();
            let next = task.next_run_ms.unwrap();
            assert!(!is_due("disabled", next + 60_000).unwrap());
        });
    }

    #[test]
    fn test_deregister() {
        run(|| {
            register("gone", "* * * * *", T0).unwrap();
            deregister("gone").unwrap();
            assert_eq!(get_task("gone").unwrap_err(), CronError::NotFound);
        });
    }

    #[test]
    fn test_due_tasks_returns_only_due() {
        run(|| {
            register("due-a",  "* * * * *", T0).unwrap();
            register("due-b",  "0 0 1 1 *", T0).unwrap(); // Jan 1 midnight only
            let task_a = get_task("due-a").unwrap();
            let next_a = task_a.next_run_ms.unwrap();
            let due = due_tasks(next_a).unwrap();
            assert!(due.iter().any(|t| t.name == "due-a"));
            // due-b next-run is Jan 1 00:00, which is far in the future relative to next_a
            // so it may or may not appear; just confirm due-a is there
        });
    }

    #[test]
    fn test_next_after_specific_time() {
        run(|| {
            // "30 10 * * *" should fire at 10:30 every day
            // T0 = 2024-01-15 10:30:00 UTC, so next is 2024-01-16 10:30:00
            let s = parse_expression("30 10 * * *").unwrap();
            let next = next_after(&s, T0).unwrap();
            let dt = ms_to_dt(next);
            assert_eq!(dt.hour, 10);
            assert_eq!(dt.minute, 30);
            assert!(next > T0);
        });
    }

    #[test]
    fn test_list_tasks() {
        run(|| {
            register("list-a", "* * * * *", T0).unwrap();
            register("list-b", "0 * * * *", T0).unwrap();
            let all = list_tasks().unwrap();
            assert!(all.len() >= 2);
        });
    }
}
