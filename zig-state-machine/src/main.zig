//! Composition root. The ONE place that sees every layer. It reads config from
//! the environment (12-factor style), constructs the chosen concrete repo
//! (in-memory or sqlite), erases it to the usecases.TodoRepo interface, and
//! hands that interface to the http dispatcher. domain, usecases and http never
//! name a concrete repo — only this file does the wiring.
//!
//! Config (all via env, with sane defaults):
//!   TODO_BACKEND  memory | sqlite   (default sqlite)
//!   TODO_DB_PATH  sqlite file path  (default todos.db; ignored for memory)
//!   HOST          bind address      (default 0.0.0.0 — containers need this)
//!   PORT          listen port       (default 8080)

const std = @import("std");
const memory = @import("memory");
const sqlite = @import("sqlite");
const http = @import("http");
const usecases = @import("usecases");

// --- Graceful shutdown plumbing ---
// A signal handler runs on an arbitrary thread with almost nothing safe to
// touch, so it only does two async-signal-safe things: set an atomic flag and
// close the listening fd. Closing the fd interrupts the blocked accept() so the
// loop wakes, sees the flag, stops accepting, and drains in-flight work.
var shutting_down = std.atomic.Value(bool).init(false);

fn onSignal(_: std.posix.SIG, _: *const std.posix.siginfo_t, _: ?*anyopaque) callconv(.c) void {
    // Only thing safe in a handler: set the flag. A watcher task notices it and
    // cancels the acceptor through the io layer (the sanctioned way to interrupt
    // a blocked accept — closing the fd directly trips a BADF panic in debug).
    shutting_down.store(true, .seq_cst);
}

fn installSignalHandlers() void {
    var act = std.posix.Sigaction{
        .handler = .{ .sigaction = onSignal },
        .mask = std.posix.sigemptyset(),
        .flags = std.posix.SA.SIGINFO,
    };
    std.posix.sigaction(.INT, &act, null);
    std.posix.sigaction(.TERM, &act, null);
}

/// Holds whichever concrete repo we built so it outlives the accept loop, and
/// hands out the type-erased interface. Adding a backend = one more variant.
const Backend = union(enum) {
    memory: memory.MemoryRepo,
    sqlite: sqlite.SqliteRepo,

    fn repo(self: *Backend) usecases.TodoRepo {
        return switch (self.*) {
            .memory => |*m| m.repo(),
            .sqlite => |*s| s.repo(),
        };
    }
    fn deinit(self: *Backend) void {
        switch (self.*) {
            inline else => |*r| r.deinit(),
        }
    }
};

pub fn main(init: std.process.Init) !void {
    const io = init.io;
    const gpa = init.arena.allocator(); // process-lifetime allocator
    const env = init.environ_map;

    const backend_name = env.get("TODO_BACKEND") orelse "sqlite";
    const db_path_raw = env.get("TODO_DB_PATH") orelse "todos.db";
    const host = env.get("HOST") orelse "0.0.0.0";
    const port: u16 = if (env.get("PORT")) |p|
        std.fmt.parseInt(u16, p, 10) catch 8080
    else
        8080;

    // --- Build the chosen backend; both expose the same TodoRepo interface. ---
    var backend: Backend = if (std.mem.eql(u8, backend_name, "memory"))
        .{ .memory = memory.MemoryRepo.init(gpa) }
    else
        .{ .sqlite = try sqlite.SqliteRepo.open(gpa, try gpa.dupeSentinel(u8, db_path_raw, 0)) };
    defer backend.deinit();
    const repo = backend.repo();

    // --- Listen. ---
    const addr = try std.Io.net.IpAddress.parse(host, port);
    var server = try addr.listen(io, .{ .reuse_address = true });
    defer server.deinit(io);

    installSignalHandlers();

    var stdout_buf: [80]u8 = undefined;
    var stdout: std.Io.File.Writer = .init(.stdout(), io, &stdout_buf);
    try stdout.interface.print("listening on {s}:{d} (backend={s})\n", .{ host, port, backend_name });
    try stdout.interface.flush();

    // Two task groups:
    //  * accept_group runs the single acceptor task. Cancelling it interrupts a
    //    blocked accept() through the io layer — the clean way to wake it.
    //  * conn_group tracks in-flight connections, each handled concurrently so a
    //    slow client can't block others.
    var conn_group: std.Io.Group = .init;
    var accept_group: std.Io.Group = .init;
    accept_group.concurrent(io, acceptLoop, .{ io, &server, repo, gpa, &conn_group }) catch {
        // No concurrency available: fall back to a serial inline loop.
        acceptLoop(io, &server, repo, gpa, &conn_group);
        return;
    };

    // Watcher: wait for a shutdown signal, then cancel the acceptor and drain.
    while (!shutting_down.load(.seq_cst)) {
        std.Io.sleep(io, .fromMilliseconds(100), .awake) catch break;
    }

    try stdout.interface.print("shutting down, draining in-flight requests...\n", .{});
    try stdout.interface.flush();
    accept_group.cancel(io); // interrupt the blocked accept()
    conn_group.await(io) catch {}; // let outstanding requests finish
    // defers run: server.deinit (close listener), backend.deinit (flush+close db).
}

/// Accept connections until cancelled, handing each to a concurrent task.
fn acceptLoop(
    io: std.Io,
    server: *std.Io.net.Server,
    repo: usecases.TodoRepo,
    gpa: std.mem.Allocator,
    conn_group: *std.Io.Group,
) void {
    while (true) {
        const stream = server.accept(io) catch return; // cancellation lands here
        conn_group.concurrent(io, handleConn, .{ io, repo, gpa, stream }) catch {
            handleConn(io, repo, gpa, stream); // inline fallback
        };
    }
}

/// Handle one connection: serve a single request, then close. Owns its own
/// arena so concurrent connections never share allocator state.
fn handleConn(
    io: std.Io,
    repo: usecases.TodoRepo,
    gpa: std.mem.Allocator,
    stream: std.Io.net.Stream,
) void {
    defer stream.close(io);

    var arena = std.heap.ArenaAllocator.init(gpa);
    defer arena.deinit();

    var in_buf: [8192]u8 = undefined;
    var out_buf: [8192]u8 = undefined;
    var reader = stream.reader(io, &in_buf);
    var writer = stream.writer(io, &out_buf);

    var hs = std.http.Server.init(&reader.interface, &writer.interface);

    // HTTP/1.1 keep-alive: serve requests on this connection until the client
    // closes it (receiveHead returns an error) or we fail to respond.
    while (true) {
        var req = hs.receiveHead() catch return; // EndOfStream / closed by peer
        http.handle(arena.allocator(), repo, &req) catch return;
        writer.interface.flush() catch return;
        _ = arena.reset(.retain_capacity);
    }
}
