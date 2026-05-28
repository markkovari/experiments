//! Domain layer — pure entities and rules. Depends on `std` only.
//! No I/O, no persistence, no allocation of owned state. Callers own strings.

const std = @import("std");

/// Lifecycle state. A Todo moves todo -> in_progress -> done, with reopen back.
pub const Status = enum {
    todo,
    in_progress,
    done,

    pub fn isTerminal(self: Status) bool {
        return self == .done;
    }
};

/// Errors that express broken domain invariants.
pub const DomainError = error{
    TitleEmpty,
    TitleTooLong,
    InvalidTransition,
};

pub const max_title_len = 200;

/// Core entity. Idless — identity is a storage concern, assigned at the repo
/// edge (see usecases `Stored(Todo)`). `title` is a borrowed slice; caller owns it.
pub const Todo = struct {
    title: []const u8,
    status: Status,

    /// Construct a valid Todo or fail an invariant. Does not allocate.
    pub fn init(title: []const u8) DomainError!Todo {
        try validateTitle(title);
        return .{ .title = title, .status = .todo };
    }

    /// Begin work. Only valid from `todo`.
    pub fn start(self: *Todo) DomainError!void {
        try self.transition(.in_progress);
    }

    /// Mark complete. Valid from `todo` or `in_progress`.
    pub fn complete(self: *Todo) DomainError!void {
        try self.transition(.done);
    }

    /// Move back to `todo`. Valid from any state.
    pub fn reopen(self: *Todo) void {
        self.status = .todo;
    }

    /// Rename with validation. New slice still borrowed from caller.
    pub fn rename(self: *Todo, title: []const u8) DomainError!void {
        try validateTitle(title);
        self.title = title;
    }

    /// Enforce the allowed state graph.
    fn transition(self: *Todo, next: Status) DomainError!void {
        const ok = switch (self.status) {
            .todo => next == .in_progress or next == .done,
            .in_progress => next == .done,
            .done => false,
        };
        if (!ok) return error.InvalidTransition;
        self.status = next;
    }
};

fn validateTitle(title: []const u8) DomainError!void {
    if (title.len == 0) return error.TitleEmpty;
    if (title.len > max_title_len) return error.TitleTooLong;
}

test "init rejects empty title" {
    try std.testing.expectError(error.TitleEmpty, Todo.init(""));
}

test "init rejects oversized title" {
    const oversized = comptime blk: {
        var buf: [max_title_len + 1]u8 = undefined;
        for (&buf) |*c| c.* = 'x';
        break :blk buf;
    };
    try std.testing.expectError(error.TitleTooLong, Todo.init(&oversized));
}

test "happy path: todo -> in_progress -> done" {
    var t = try Todo.init("buy milk");
    try std.testing.expectEqual(Status.todo, t.status);
    try t.start();
    try std.testing.expectEqual(Status.in_progress, t.status);
    try t.complete();
    try std.testing.expectEqual(Status.done, t.status);
    try std.testing.expect(t.status.isTerminal());
}

test "cannot start a done todo" {
    var t = try Todo.init("x");
    try t.complete();
    try std.testing.expectError(error.InvalidTransition, t.start());
}

test "reopen from done returns to todo" {
    var t = try Todo.init("x");
    try t.complete();
    t.reopen();
    try std.testing.expectEqual(Status.todo, t.status);
}
