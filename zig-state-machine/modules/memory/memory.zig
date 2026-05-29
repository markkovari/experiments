//! In-memory TodoRepo implementation. A real (non-test) concrete repo that
//! fills the usecases.TodoRepo vtable. Owns its todo titles (dupes on insert)
//! so callers may free their request buffers. Depends on usecases (for the
//! interface + types) and domain (transitively). Never touches http.
//!
//! This is exactly the shape sqlite will take later: fill the vtable, @ptrCast
//! the erased self back, do the real storage work. Swapping memory -> sqlite
//! changes only the composition root.

const std = @import("std");
const usecases = @import("usecases");

const Todo = usecases.Todo;
const StoredTodo = usecases.StoredTodo;
const TodoId = usecases.TodoId;
const TodoRepo = usecases.TodoRepo;
const RepoError = usecases.RepoError;

pub const MemoryRepo = struct {
    alloc: std.mem.Allocator,
    items: std.ArrayList(StoredTodo),
    next_id: TodoId = 1,

    pub fn init(alloc: std.mem.Allocator) MemoryRepo {
        return .{ .alloc = alloc, .items = .empty };
    }

    /// Frees every owned title plus the backing list.
    pub fn deinit(self: *MemoryRepo) void {
        for (self.items.items) |rec| self.alloc.free(rec.value.title);
        self.items.deinit(self.alloc);
    }

    /// Package self + the shared vtable into the storage-agnostic interface.
    pub fn repo(self: *MemoryRepo) TodoRepo {
        return .{ .ptr = self, .vtable = &vtable };
    }

    const vtable = TodoRepo.VTable{
        .add = add,
        .get = get,
        .update = update,
        .list = list,
    };

    fn add(ptr: *anyopaque, todo: Todo) RepoError!StoredTodo {
        const self: *MemoryRepo = @ptrCast(@alignCast(ptr));
        // Own the title: domain borrows it, our store outlives the caller's buffer.
        const owned = self.alloc.dupe(u8, todo.title) catch return error.OutOfMemory;
        errdefer self.alloc.free(owned);
        const rec = StoredTodo{
            .id = self.next_id,
            .value = .{ .title = owned, .status = todo.status },
        };
        self.items.append(self.alloc, rec) catch {
            self.alloc.free(owned);
            return error.OutOfMemory;
        };
        self.next_id += 1;
        return rec;
    }

    fn get(ptr: *anyopaque, id: TodoId) RepoError!StoredTodo {
        const self: *MemoryRepo = @ptrCast(@alignCast(ptr));
        for (self.items.items) |rec| {
            if (rec.id == id) return rec;
        }
        return error.NotFound;
    }

    fn update(ptr: *anyopaque, id: TodoId, todo: Todo) RepoError!void {
        const self: *MemoryRepo = @ptrCast(@alignCast(ptr));
        for (self.items.items) |*rec| {
            if (rec.id == id) {
                // Title may have changed (rename); re-own if different pointer.
                if (rec.value.title.ptr != todo.title.ptr) {
                    const owned = self.alloc.dupe(u8, todo.title) catch return error.OutOfMemory;
                    self.alloc.free(rec.value.title);
                    rec.value = .{ .title = owned, .status = todo.status };
                } else {
                    rec.value.status = todo.status;
                }
                return;
            }
        }
        return error.NotFound;
    }

    fn list(ptr: *anyopaque, alloc: std.mem.Allocator) RepoError![]StoredTodo {
        const self: *MemoryRepo = @ptrCast(@alignCast(ptr));
        return alloc.dupe(StoredTodo, self.items.items) catch error.OutOfMemory;
    }
};

test "MemoryRepo round-trips through the interface and owns titles" {
    var mem = MemoryRepo.init(std.testing.allocator);
    defer mem.deinit();
    const repo = mem.repo();

    // Title in a buffer we free immediately — repo must have copied it.
    var buf: [16]u8 = undefined;
    const title = buf[0..3];
    @memcpy(title, "abc");
    const rec = try repo.add(.{ .title = title, .status = .todo });
    @memcpy(title, "zzz"); // scribble the original; stored copy must be intact

    const got = try repo.get(rec.id);
    try std.testing.expectEqualStrings("abc", got.value.title);
}

test "update persists status change" {
    var mem = MemoryRepo.init(std.testing.allocator);
    defer mem.deinit();
    const repo = mem.repo();
    const rec = try repo.add(.{ .title = "x", .status = .todo });
    try repo.update(rec.id, .{ .title = "x", .status = .done });
    const got = try repo.get(rec.id);
    try std.testing.expectEqual(usecases.Status.done, got.value.status);
}
