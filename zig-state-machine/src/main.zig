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

    // Announce readiness on stdout so an e2e harness knows we are up.
    var stdout_buf: [64]u8 = undefined;
    var stdout: std.Io.File.Writer = .init(.stdout(), io, &stdout_buf);
    try stdout.interface.print("listening on {s}:{d} (backend={s})\n", .{ host, port, backend_name });
    try stdout.interface.flush();

    // --- Accept loop. One request per connection (keep_alive off for simplicity). ---
    // A per-request arena: http allocates freely, we reset after each response.
    var req_arena = std.heap.ArenaAllocator.init(gpa);
    defer req_arena.deinit();

    while (true) {
        const stream = server.accept(io) catch continue;
        // Serve exactly one request, then close so the client sees EOF
        // (we respond with keep_alive=false). Closing also frees the fd.
        serveOne(io, repo, req_arena.allocator(), stream);
        _ = req_arena.reset(.retain_capacity);
    }
}

fn serveOne(
    io: std.Io,
    repo: usecases.TodoRepo,
    alloc: std.mem.Allocator,
    stream: std.Io.net.Stream,
) void {
    defer stream.close(io);

    var in_buf: [8192]u8 = undefined;
    var out_buf: [8192]u8 = undefined;
    var reader = stream.reader(io, &in_buf);
    var writer = stream.writer(io, &out_buf);

    var hs = std.http.Server.init(&reader.interface, &writer.interface);
    var req = hs.receiveHead() catch return;

    http.handle(alloc, repo, &req) catch {};
    writer.interface.flush() catch {};
}
