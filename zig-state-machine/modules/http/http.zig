//! HTTP layer — maps requests to use-case calls and serializes results to JSON.
//! Depends on usecases only; never on memory/sqlite (it holds the interface,
//! not a concrete repo) and never on domain directly. The accept-loop lives in
//! main; this module is pure per-request dispatch so it stays testable and thin.
//!
//! Routes:
//!   POST /todos                 {"title": "..."} -> 201 {id,title,status}
//!   GET  /todos                              -> 200 [{id,title,status}, ...]
//!   POST /todos/{id}/start                   -> 200 {id,title,status}
//!   POST /todos/{id}/complete                -> 200 {id,title,status}

const std = @import("std");
const usecases = @import("usecases");

const TodoRepo = usecases.TodoRepo;
const StoredTodo = usecases.StoredTodo;
const Status = usecases.Status;

/// Dispatch one request against the repo. `alloc` is a per-request arena the
/// caller owns and resets; everything we allocate here dies with it.
pub fn handle(
    alloc: std.mem.Allocator,
    repo: TodoRepo,
    req: *std.http.Server.Request,
) !void {
    const method = req.head.method;
    const target = req.head.target;

    if (method == .POST and std.mem.eql(u8, target, "/todos")) {
        return createTodo(alloc, repo, req);
    }
    if (method == .GET and std.mem.eql(u8, target, "/todos")) {
        return listTodos(alloc, repo, req);
    }
    // /todos/{id}/start | /todos/{id}/complete
    if (method == .POST) {
        if (parseAction(target)) |pa| {
            return transition(alloc, repo, req, pa.id, pa.action);
        }
    }
    return respondJson(req, .not_found, "{\"error\":\"not found\"}");
}

const json_headers = &[_]std.http.Header{
    .{ .name = "content-type", .value = "application/json" },
};

/// JSON response with HTTP/1.1 keep-alive: the connection is reused for the
/// next request. std.http discards any unread request body before sending,
/// which is safe as long as the client framed it (Content-Length / chunked) —
/// all real HTTP clients do.
fn respondJson(req: *std.http.Server.Request, status: std.http.Status, body: []const u8) !void {
    return req.respond(body, .{
        .status = status,
        .keep_alive = true,
        .extra_headers = json_headers,
    });
}

const Action = enum { start, complete };
const ParsedAction = struct { id: usecases.TodoId, action: Action };

/// Match "/todos/{id}/start" or "/todos/{id}/complete". Returns null otherwise.
fn parseAction(target: []const u8) ?ParsedAction {
    const prefix = "/todos/";
    if (!std.mem.startsWith(u8, target, prefix)) return null;
    const rest = target[prefix.len..]; // "{id}/start"
    const slash = std.mem.indexOfScalar(u8, rest, '/') orelse return null;
    const id = std.fmt.parseInt(usecases.TodoId, rest[0..slash], 10) catch return null;
    const verb = rest[slash + 1 ..];
    const action: Action = if (std.mem.eql(u8, verb, "start"))
        .start
    else if (std.mem.eql(u8, verb, "complete"))
        .complete
    else
        return null;
    return .{ .id = id, .action = action };
}

const CreateBody = struct { title: []const u8 };

fn createTodo(alloc: std.mem.Allocator, repo: TodoRepo, req: *std.http.Server.Request) !void {
    var body_buf: [4096]u8 = undefined;
    const reader = req.readerExpectContinue(&body_buf) catch
        return badRequest(req, "cannot read body");
    const raw = reader.allocRemaining(alloc, .limited(4096)) catch
        return badRequest(req, "body too large");

    const parsed = std.json.parseFromSlice(CreateBody, alloc, raw, .{}) catch
        return badRequest(req, "invalid json");
    defer parsed.deinit();

    const rec = usecases.addTodo(repo, parsed.value.title) catch |err| switch (err) {
        error.TitleEmpty => return badRequest(req, "title required"),
        error.TitleTooLong => return badRequest(req, "title too long"),
        else => return serverError(req),
    };
    const json = try todoJson(alloc, rec);
    return respondJson(req, .created, json);
}

fn listTodos(alloc: std.mem.Allocator, repo: TodoRepo, req: *std.http.Server.Request) !void {
    const all = usecases.listTodos(repo, alloc) catch return serverError(req);
    var out = std.ArrayList(u8).empty;
    try out.append(alloc, '[');
    for (all, 0..) |rec, i| {
        if (i != 0) try out.append(alloc, ',');
        const j = try todoJson(alloc, rec);
        try out.appendSlice(alloc, j);
    }
    try out.append(alloc, ']');
    return respondJson(req, .ok, out.items);
}

fn transition(
    alloc: std.mem.Allocator,
    repo: TodoRepo,
    req: *std.http.Server.Request,
    id: usecases.TodoId,
    action: Action,
) !void {
    const rec = switch (action) {
        .start => usecases.startTodo(repo, id),
        .complete => usecases.completeTodo(repo, id),
    } catch |err| switch (err) {
        error.NotFound => return respondJson(req, .not_found, "{\"error\":\"not found\"}"),
        error.InvalidTransition => return respondJson(req, .conflict, "{\"error\":\"invalid transition\"}"),
        else => return serverError(req),
    };
    const json = try todoJson(alloc, rec);
    return respondJson(req, .ok, json);
}

fn statusStr(s: Status) []const u8 {
    return switch (s) {
        .todo => "todo",
        .in_progress => "in_progress",
        .done => "done",
    };
}

/// Serialize one stored todo to JSON in the arena.
fn todoJson(alloc: std.mem.Allocator, rec: StoredTodo) ![]u8 {
    return std.fmt.allocPrint(
        alloc,
        "{{\"id\":{d},\"title\":\"{s}\",\"status\":\"{s}\"}}",
        .{ rec.id, rec.value.title, statusStr(rec.value.status) },
    );
}

fn badRequest(req: *std.http.Server.Request, msg: []const u8) !void {
    _ = msg;
    return respondJson(req, .bad_request, "{\"error\":\"bad request\"}");
}

fn serverError(req: *std.http.Server.Request) !void {
    return respondJson(req, .internal_server_error, "{\"error\":\"internal\"}");
}

test "parseAction matches start and complete" {
    const a = parseAction("/todos/7/start").?;
    try std.testing.expectEqual(@as(usecases.TodoId, 7), a.id);
    try std.testing.expectEqual(Action.start, a.action);
    const b = parseAction("/todos/42/complete").?;
    try std.testing.expectEqual(Action.complete, b.action);
    try std.testing.expect(parseAction("/todos") == null);
    try std.testing.expect(parseAction("/todos/x/start") == null);
    try std.testing.expect(parseAction("/todos/7/bogus") == null);
}
