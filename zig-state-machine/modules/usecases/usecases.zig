//! Use-case layer — application logic. Orchestrates domain entities through a
//! storage-agnostic repository interface. Depends on `domain` only; never on
//! sqlite or http. Concrete repos are injected at the composition root (main).

const std = @import("std");
const domain = @import("domain");

pub const Todo = domain.Todo;
pub const Status = domain.Status;

/// Storage-assigned identity. Generation lives at the repo edge; the domain
/// stays idless. Any persisted value is wrapped as `Stored(T)`.
pub const TodoId = u64;

pub fn Stored(comptime T: type) type {
    return struct {
        id: TodoId,
        value: T,
    };
}

pub const StoredTodo = Stored(Todo);

/// Faults the storage backend can raise, independent of any concrete driver.
pub const RepoError = error{
    NotFound,
    StorageFailure,
    OutOfMemory,
};

/// Errors a use case may surface: domain invariant breaks plus storage faults.
pub const UseCaseError = domain.DomainError || RepoError;

// --- The interface: a hand-built vtable. {ptr, vtable} fat pointer. ---
// usecases defines the contract; outer layers implement it; main injects the
// concrete one. Same shape as std.mem.Allocator. usecases never names a repo.

pub const TodoRepo = struct {
    /// Type-erased receiver — points at the concrete repo's state.
    ptr: *anyopaque,
    /// The function table the concrete repo filled in.
    vtable: *const VTable,

    /// The contract: function pointers every concrete repo must supply.
    /// First param is the erased `self`; impls @ptrCast it back to their type.
    pub const VTable = struct {
        add: *const fn (ptr: *anyopaque, todo: Todo) RepoError!StoredTodo,
        get: *const fn (ptr: *anyopaque, id: TodoId) RepoError!StoredTodo,
        update: *const fn (ptr: *anyopaque, id: TodoId, todo: Todo) RepoError!void,
        list: *const fn (ptr: *anyopaque, alloc: std.mem.Allocator) RepoError![]StoredTodo,
    };

    // Wrapper methods — sugar that forwards self.ptr + self.vtable so callers
    // write `repo.add(t)` instead of `repo.vtable.add(repo.ptr, t)`.
    pub fn add(self: TodoRepo, todo: Todo) RepoError!StoredTodo {
        return self.vtable.add(self.ptr, todo);
    }
    pub fn get(self: TodoRepo, id: TodoId) RepoError!StoredTodo {
        return self.vtable.get(self.ptr, id);
    }
    pub fn update(self: TodoRepo, id: TodoId, todo: Todo) RepoError!void {
        return self.vtable.update(self.ptr, id, todo);
    }
    pub fn list(self: TodoRepo, alloc: std.mem.Allocator) RepoError![]StoredTodo {
        return self.vtable.list(self.ptr, alloc);
    }
};

// --- Use cases: thin orchestration of domain rules over the repo. ---

/// Create a valid todo and persist it. Validation runs before storage.
pub fn addTodo(repo: TodoRepo, title: []const u8) UseCaseError!StoredTodo {
    const todo = try Todo.init(title);
    return repo.add(todo);
}

/// Move a todo to in_progress, persisting the transition.
pub fn startTodo(repo: TodoRepo, id: TodoId) UseCaseError!StoredTodo {
    var rec = try repo.get(id);
    try rec.value.start();
    try repo.update(id, rec.value);
    return rec;
}

/// Complete a todo, persisting the transition.
pub fn completeTodo(repo: TodoRepo, id: TodoId) UseCaseError!StoredTodo {
    var rec = try repo.get(id);
    try rec.value.complete();
    try repo.update(id, rec.value);
    return rec;
}

/// List all stored todos. Caller owns and frees the returned slice.
pub fn listTodos(repo: TodoRepo, alloc: std.mem.Allocator) UseCaseError![]StoredTodo {
    return repo.list(alloc);
}

// --- Tests use a fake in-memory repo, proving the interface alone suffices. ---
// This fake is the concrete impl that fills the vtable and @ptrCasts back —
// exactly what sqlite will do later.

const FakeRepo = struct {
    alloc: std.mem.Allocator,
    items: std.ArrayList(StoredTodo),
    next_id: TodoId = 1,

    fn init(alloc: std.mem.Allocator) FakeRepo {
        return .{ .alloc = alloc, .items = .empty };
    }
    fn deinit(self: *FakeRepo) void {
        self.items.deinit(self.alloc);
    }

    /// Package self + the shared vtable into a TodoRepo. .ptr = self erases
    /// *FakeRepo -> *anyopaque automatically (erasing is always safe).
    fn repo(self: *FakeRepo) TodoRepo {
        return .{ .ptr = self, .vtable = &vtable };
    }

    // One vtable, shared by all instances. Built at comptime.
    const vtable = TodoRepo.VTable{
        .add = add,
        .get = get,
        .update = update,
        .list = list,
    };

    fn add(ptr: *anyopaque, todo: Todo) RepoError!StoredTodo {
        const self: *FakeRepo = @ptrCast(@alignCast(ptr)); // re-attach the type
        const rec = StoredTodo{ .id = self.next_id, .value = todo };
        self.items.append(self.alloc, rec) catch return error.OutOfMemory;
        self.next_id += 1;
        return rec;
    }
    fn get(ptr: *anyopaque, id: TodoId) RepoError!StoredTodo {
        const self: *FakeRepo = @ptrCast(@alignCast(ptr));
        for (self.items.items) |rec| {
            if (rec.id == id) return rec;
        }
        return error.NotFound;
    }
    fn update(ptr: *anyopaque, id: TodoId, todo: Todo) RepoError!void {
        const self: *FakeRepo = @ptrCast(@alignCast(ptr));
        for (self.items.items) |*rec| {
            if (rec.id == id) {
                rec.value = todo;
                return;
            }
        }
        return error.NotFound;
    }
    fn list(ptr: *anyopaque, alloc: std.mem.Allocator) RepoError![]StoredTodo {
        const self: *FakeRepo = @ptrCast(@alignCast(ptr));
        return alloc.dupe(StoredTodo, self.items.items) catch error.OutOfMemory;
    }
};

test "addTodo persists and assigns an id" {
    var fake = FakeRepo.init(std.testing.allocator);
    defer fake.deinit();
    const repo = fake.repo();

    const rec = try addTodo(repo, "buy milk");
    try std.testing.expectEqual(@as(TodoId, 1), rec.id);
    try std.testing.expectEqual(Status.todo, rec.value.status);
}

test "addTodo rejects invalid title before touching storage" {
    var fake = FakeRepo.init(std.testing.allocator);
    defer fake.deinit();
    try std.testing.expectError(error.TitleEmpty, addTodo(fake.repo(), ""));
    try std.testing.expectEqual(@as(usize, 0), fake.items.items.len);
}

test "startTodo then completeTodo persist transitions" {
    var fake = FakeRepo.init(std.testing.allocator);
    defer fake.deinit();
    const repo = fake.repo();

    const created = try addTodo(repo, "task");
    _ = try startTodo(repo, created.id);
    const done = try completeTodo(repo, created.id);
    try std.testing.expectEqual(Status.done, done.value.status);

    const reread = try repo.get(created.id);
    try std.testing.expectEqual(Status.done, reread.value.status);
}

test "completeTodo on missing id is NotFound" {
    var fake = FakeRepo.init(std.testing.allocator);
    defer fake.deinit();
    try std.testing.expectError(error.NotFound, completeTodo(fake.repo(), 999));
}

test "listTodos returns all, caller frees" {
    var fake = FakeRepo.init(std.testing.allocator);
    defer fake.deinit();
    const repo = fake.repo();
    _ = try addTodo(repo, "a");
    _ = try addTodo(repo, "b");

    const all = try listTodos(repo, std.testing.allocator);
    defer std.testing.allocator.free(all);
    try std.testing.expectEqual(@as(usize, 2), all.len);
}
