//! End-to-end test. Spawns the actual built todo binary as a child process,
//! waits for its port to open, then drives it over real TCP with hand-written
//! HTTP/1.1 requests and asserts on the raw responses. Exercises the whole
//! stack — main wiring, http routing, usecases, domain, memory repo — exactly
//! as a deployed server would be hit. No mocks below the socket.

const std = @import("std");
const testing = std.testing;

const port = 8123;
const exe_path = "zig-out/bin/zig_state_machine";

const io = testing.io;

/// Open a fresh connection, send `request`, return the full response bytes.
/// Caller frees. One connection per call — the server runs keep_alive=false.
fn roundtrip(alloc: std.mem.Allocator, request: []const u8) ![]u8 {
    const addr = try std.Io.net.IpAddress.parse("127.0.0.1", port);
    const stream = try addr.connect(io, .{ .mode = .stream });
    defer stream.close(io);

    var wbuf: [4096]u8 = undefined;
    var writer = stream.writer(io, &wbuf);
    try writer.interface.writeAll(request);
    try writer.interface.flush();

    var rbuf: [8192]u8 = undefined;
    var reader = stream.reader(io, &rbuf);
    // Server closes the connection after responding, so read to EOF.
    return reader.interface.allocRemaining(alloc, .limited(8192));
}

fn get(alloc: std.mem.Allocator, path: []const u8) ![]u8 {
    const req = try std.fmt.allocPrint(alloc,
        "GET {s} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n", .{path});
    defer alloc.free(req);
    return roundtrip(alloc, req);
}

fn post(alloc: std.mem.Allocator, path: []const u8, body: []const u8) ![]u8 {
    const req = try std.fmt.allocPrint(alloc,
        "POST {s} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: {d}\r\n\r\n{s}",
        .{ path, body.len, body });
    defer alloc.free(req);
    return roundtrip(alloc, req);
}

/// Parse the numeric status code out of "HTTP/1.1 NNN ...".
fn statusOf(response: []const u8) !u16 {
    var it = std.mem.tokenizeScalar(u8, response, ' ');
    _ = it.next() orelse return error.BadResponse; // "HTTP/1.1"
    const code = it.next() orelse return error.BadResponse;
    return std.fmt.parseInt(u16, code, 10);
}

fn bodyOf(response: []const u8) []const u8 {
    const sep = std.mem.indexOf(u8, response, "\r\n\r\n") orelse return "";
    return response[sep + 4 ..];
}

test "e2e: full todo lifecycle over real HTTP" {
    const alloc = testing.allocator;

    // --- Spawn the real server. ---
    var child = try std.process.spawn(io, .{
        .argv = &.{ exe_path, comptimePort() },
        .stdout = .ignore,
        .stderr = .ignore,
    });
    defer child.kill(io); // kill reaps and cleans up; no separate wait needed

    // --- Wait for the port to accept connections (retry up to ~2s). ---
    try waitForPort(alloc);

    // --- create ---
    {
        const res = try post(alloc, "/todos", "{\"title\":\"buy milk\"}");
        defer alloc.free(res);
        try testing.expectEqual(@as(u16, 201), try statusOf(res));
        try testing.expect(std.mem.indexOf(u8, bodyOf(res), "\"status\":\"todo\"") != null);
        try testing.expect(std.mem.indexOf(u8, bodyOf(res), "\"id\":1") != null);
    }

    // --- start ---
    {
        const res = try post(alloc, "/todos/1/start", "");
        defer alloc.free(res);
        try testing.expectEqual(@as(u16, 200), try statusOf(res));
        try testing.expect(std.mem.indexOf(u8, bodyOf(res), "in_progress") != null);
    }

    // --- complete ---
    {
        const res = try post(alloc, "/todos/1/complete", "");
        defer alloc.free(res);
        try testing.expectEqual(@as(u16, 200), try statusOf(res));
        try testing.expect(std.mem.indexOf(u8, bodyOf(res), "\"status\":\"done\"") != null);
    }

    // --- illegal transition: start an already-done todo -> 409 ---
    {
        const res = try post(alloc, "/todos/1/start", "");
        defer alloc.free(res);
        try testing.expectEqual(@as(u16, 409), try statusOf(res));
    }

    // --- missing id -> 404 ---
    {
        const res = try post(alloc, "/todos/999/start", "");
        defer alloc.free(res);
        try testing.expectEqual(@as(u16, 404), try statusOf(res));
    }

    // --- invalid title -> 400, storage untouched ---
    {
        const res = try post(alloc, "/todos", "{\"title\":\"\"}");
        defer alloc.free(res);
        try testing.expectEqual(@as(u16, 400), try statusOf(res));
    }

    // --- list reflects the single completed todo ---
    {
        const res = try get(alloc, "/todos");
        defer alloc.free(res);
        try testing.expectEqual(@as(u16, 200), try statusOf(res));
        const body = bodyOf(res);
        try testing.expect(std.mem.indexOf(u8, body, "buy milk") != null);
        try testing.expect(std.mem.indexOf(u8, body, "done") != null);
    }
}

fn comptimePort() []const u8 {
    return std.fmt.comptimePrint("{d}", .{port});
}

fn waitForPort(alloc: std.mem.Allocator) !void {
    const addr = try std.Io.net.IpAddress.parse("127.0.0.1", port);
    var attempt: usize = 0;
    while (attempt < 40) : (attempt += 1) {
        const stream = addr.connect(io, .{ .mode = .stream }) catch {
            std.Io.sleep(io, .fromMilliseconds(50), .awake) catch {};
            continue;
        };
        stream.close(io);
        return;
    }
    _ = alloc;
    return error.ServerNeverCameUp;
}
