//! SQLite-backed TodoRepo. Fills the SAME usecases.TodoRepo vtable the
//! in-memory repo does — so swapping memory -> sqlite touches only the
//! composition root (main). Persists to a .db file on disk.
//!
//! The sqlite3 C amalgamation is compiled into the binary (see build.zig) and
//! imported here via @cImport. This module is the only place C is touched;
//! everything above it sees pure Zig types.

const std = @import("std");
const usecases = @import("usecases");

// Minimal hand-written FFI to the vendored sqlite3 amalgamation. This build of
// Zig dropped @cImport, so we declare exactly the symbols we use as `extern`.
// The C source is compiled into the binary by build.zig; the linker resolves
// these. Opaque DB/stmt handles are just *anyopaque to us.
const c = struct {
    const sqlite3 = anyopaque;
    const sqlite3_stmt = anyopaque;
    /// Bind-text destructor sentinel: SQLITE_TRANSIENT == (void*)-1 tells sqlite
    /// to copy the bound bytes, so our borrowed slices need not outlive the call.
    /// Typed as an opaque ptr (not a real fn ptr) to dodge alignment checks.
    const SQLITE_TRANSIENT: ?*const anyopaque =
        @ptrFromInt(@as(usize, @bitCast(@as(isize, -1))));

    const SQLITE_OK = 0;
    const SQLITE_ROW = 100;
    const SQLITE_DONE = 101;

    extern fn sqlite3_open(filename: [*:0]const u8, db: *?*sqlite3) c_int;
    extern fn sqlite3_close(db: ?*sqlite3) c_int;
    extern fn sqlite3_exec(db: ?*sqlite3, sql: [*:0]const u8, cb: ?*anyopaque, arg: ?*anyopaque, errmsg: ?*?[*:0]u8) c_int;
    extern fn sqlite3_prepare_v2(db: ?*sqlite3, sql: [*]const u8, n: c_int, stmt: *?*sqlite3_stmt, tail: ?*?[*]const u8) c_int;
    extern fn sqlite3_finalize(stmt: ?*sqlite3_stmt) c_int;
    extern fn sqlite3_step(stmt: ?*sqlite3_stmt) c_int;
    extern fn sqlite3_reset(stmt: ?*sqlite3_stmt) c_int;
    extern fn sqlite3_changes(db: ?*sqlite3) c_int;
    extern fn sqlite3_last_insert_rowid(db: ?*sqlite3) i64;
    extern fn sqlite3_bind_text(stmt: ?*sqlite3_stmt, idx: c_int, text: [*]const u8, n: c_int, destroy: ?*const anyopaque) c_int;
    extern fn sqlite3_bind_int64(stmt: ?*sqlite3_stmt, idx: c_int, val: i64) c_int;
    extern fn sqlite3_column_int64(stmt: ?*sqlite3_stmt, col: c_int) i64;
    extern fn sqlite3_column_text(stmt: ?*sqlite3_stmt, col: c_int) ?[*]const u8;
    extern fn sqlite3_column_bytes(stmt: ?*sqlite3_stmt, col: c_int) c_int;
};

const Todo = usecases.Todo;
const StoredTodo = usecases.StoredTodo;
const TodoId = usecases.TodoId;
const TodoRepo = usecases.TodoRepo;
const RepoError = usecases.RepoError;
const Status = usecases.Status;

pub const SqliteRepo = struct {
    alloc: std.mem.Allocator,
    db: ?*c.sqlite3,

    pub const OpenError = error{ OpenFailed, MigrateFailed };

    /// Open (or create) the database file and ensure the schema exists.
    /// `path` must be null-terminated (e.g. "todos.db" or ":memory:").
    pub fn open(alloc: std.mem.Allocator, path: [:0]const u8) OpenError!SqliteRepo {
        var db: ?*c.sqlite3 = null;
        if (c.sqlite3_open(path.ptr, &db) != c.SQLITE_OK) {
            if (db) |d| _ = c.sqlite3_close(d);
            return error.OpenFailed;
        }
        const self = SqliteRepo{ .alloc = alloc, .db = db };
        self.exec(
            \\CREATE TABLE IF NOT EXISTS todos(
            \\  id INTEGER PRIMARY KEY AUTOINCREMENT,
            \\  title TEXT NOT NULL,
            \\  status TEXT NOT NULL
            \\);
        ) catch return error.MigrateFailed;
        return self;
    }

    pub fn deinit(self: *SqliteRepo) void {
        _ = c.sqlite3_close(self.db);
    }

    pub fn repo(self: *SqliteRepo) TodoRepo {
        return .{ .ptr = self, .vtable = &vtable };
    }

    const vtable = TodoRepo.VTable{
        .add = add,
        .get = get,
        .update = update,
        .list = list,
    };

    // --- helpers ---

    /// Run a statement with no result rows and no bindings.
    fn exec(self: SqliteRepo, sql: [:0]const u8) RepoError!void {
        if (c.sqlite3_exec(self.db, sql.ptr, null, null, null) != c.SQLITE_OK) {
            return error.StorageFailure;
        }
    }

    fn statusToStr(s: Status) [:0]const u8 {
        return switch (s) {
            .todo => "todo",
            .in_progress => "in_progress",
            .done => "done",
        };
    }

    fn strToStatus(s: []const u8) Status {
        if (std.mem.eql(u8, s, "in_progress")) return .in_progress;
        if (std.mem.eql(u8, s, "done")) return .done;
        return .todo;
    }

    // --- vtable impls (ptr is the erased *SqliteRepo) ---

    fn add(ptr: *anyopaque, todo: Todo) RepoError!StoredTodo {
        const self: *SqliteRepo = @ptrCast(@alignCast(ptr));

        var stmt: ?*c.sqlite3_stmt = null;
        if (c.sqlite3_prepare_v2(self.db, "INSERT INTO todos(title,status) VALUES(?,?)", -1, &stmt, null) != c.SQLITE_OK)
            return error.StorageFailure;
        defer _ = c.sqlite3_finalize(stmt);

        _ = c.sqlite3_bind_text(stmt, 1, todo.title.ptr, @intCast(todo.title.len), c.SQLITE_TRANSIENT);
        const st = statusToStr(todo.status);
        _ = c.sqlite3_bind_text(stmt, 2, st.ptr, @intCast(st.len), c.SQLITE_TRANSIENT);

        if (c.sqlite3_step(stmt) != c.SQLITE_DONE) return error.StorageFailure;

        const id: TodoId = @intCast(c.sqlite3_last_insert_rowid(self.db));
        // Return a stored copy whose title we own in the caller's arena.
        const owned = self.alloc.dupe(u8, todo.title) catch return error.OutOfMemory;
        return .{ .id = id, .value = .{ .title = owned, .status = todo.status } };
    }

    fn get(ptr: *anyopaque, id: TodoId) RepoError!StoredTodo {
        const self: *SqliteRepo = @ptrCast(@alignCast(ptr));

        var stmt: ?*c.sqlite3_stmt = null;
        if (c.sqlite3_prepare_v2(self.db, "SELECT title,status FROM todos WHERE id=?", -1, &stmt, null) != c.SQLITE_OK)
            return error.StorageFailure;
        defer _ = c.sqlite3_finalize(stmt);

        _ = c.sqlite3_bind_int64(stmt, 1, @intCast(id));
        switch (c.sqlite3_step(stmt)) {
            c.SQLITE_ROW => return self.rowToStored(stmt, id),
            c.SQLITE_DONE => return error.NotFound,
            else => return error.StorageFailure,
        }
    }

    fn update(ptr: *anyopaque, id: TodoId, todo: Todo) RepoError!void {
        const self: *SqliteRepo = @ptrCast(@alignCast(ptr));

        var stmt: ?*c.sqlite3_stmt = null;
        if (c.sqlite3_prepare_v2(self.db, "UPDATE todos SET title=?,status=? WHERE id=?", -1, &stmt, null) != c.SQLITE_OK)
            return error.StorageFailure;
        defer _ = c.sqlite3_finalize(stmt);

        _ = c.sqlite3_bind_text(stmt, 1, todo.title.ptr, @intCast(todo.title.len), c.SQLITE_TRANSIENT);
        const st = statusToStr(todo.status);
        _ = c.sqlite3_bind_text(stmt, 2, st.ptr, @intCast(st.len), c.SQLITE_TRANSIENT);
        _ = c.sqlite3_bind_int64(stmt, 3, @intCast(id));

        if (c.sqlite3_step(stmt) != c.SQLITE_DONE) return error.StorageFailure;
        if (c.sqlite3_changes(self.db) == 0) return error.NotFound;
    }

    fn list(ptr: *anyopaque, alloc: std.mem.Allocator) RepoError![]StoredTodo {
        const self: *SqliteRepo = @ptrCast(@alignCast(ptr));

        var stmt: ?*c.sqlite3_stmt = null;
        if (c.sqlite3_prepare_v2(self.db, "SELECT id,title,status FROM todos ORDER BY id", -1, &stmt, null) != c.SQLITE_OK)
            return error.StorageFailure;
        defer _ = c.sqlite3_finalize(stmt);

        var out = std.ArrayList(StoredTodo).empty;
        errdefer {
            for (out.items) |rec| alloc.free(rec.value.title);
            out.deinit(alloc);
        }
        while (true) {
            switch (c.sqlite3_step(stmt)) {
                c.SQLITE_ROW => {
                    const id: TodoId = @intCast(c.sqlite3_column_int64(stmt, 0));
                    const rec = try columnRowToStored(alloc, stmt, id, 1, 2);
                    out.append(alloc, rec) catch {
                        alloc.free(rec.value.title);
                        return error.OutOfMemory;
                    };
                },
                c.SQLITE_DONE => break,
                else => return error.StorageFailure,
            }
        }
        return out.toOwnedSlice(alloc) catch error.OutOfMemory;
    }

    /// Build a StoredTodo from a 2-column (title,status) row, dup'ing into self.alloc.
    fn rowToStored(self: SqliteRepo, stmt: ?*c.sqlite3_stmt, id: TodoId) RepoError!StoredTodo {
        return columnRowToStored(self.alloc, stmt, id, 0, 1);
    }
};

/// Shared row->StoredTodo: title at col `tcol`, status at col `scol`.
/// Column text pointers are invalidated by the next step/finalize, so dup now.
fn columnRowToStored(
    alloc: std.mem.Allocator,
    stmt: ?*c.sqlite3_stmt,
    id: TodoId,
    tcol: c_int,
    scol: c_int,
) RepoError!StoredTodo {
    const title_ptr = c.sqlite3_column_text(stmt, tcol) orelse return error.StorageFailure;
    const title_len: usize = @intCast(c.sqlite3_column_bytes(stmt, tcol));
    const title = title_ptr[0..title_len];

    const status_ptr = c.sqlite3_column_text(stmt, scol) orelse return error.StorageFailure;
    const status_len: usize = @intCast(c.sqlite3_column_bytes(stmt, scol));
    const status = status_ptr[0..status_len];

    const owned = alloc.dupe(u8, title) catch return error.OutOfMemory;
    return .{
        .id = id,
        .value = .{ .title = owned, .status = SqliteRepo.strToStatus(status) },
    };
}

test "sqlite repo round-trips through the interface (in-memory db)" {
    var sql = try SqliteRepo.open(std.testing.allocator, ":memory:");
    defer sql.deinit();
    const repo = sql.repo();

    const rec = try repo.add(.{ .title = "buy milk", .status = .todo });
    std.testing.allocator.free(rec.value.title);
    try std.testing.expectEqual(@as(TodoId, 1), rec.id);

    try repo.update(rec.id, .{ .title = "buy milk", .status = .done });

    const got = try repo.get(rec.id);
    defer std.testing.allocator.free(got.value.title);
    try std.testing.expectEqualStrings("buy milk", got.value.title);
    try std.testing.expectEqual(Status.done, got.value.status);

    const all = try repo.list(std.testing.allocator);
    defer {
        for (all) |r| std.testing.allocator.free(r.value.title);
        std.testing.allocator.free(all);
    }
    try std.testing.expectEqual(@as(usize, 1), all.len);

    try std.testing.expectError(error.NotFound, repo.get(999));
}
