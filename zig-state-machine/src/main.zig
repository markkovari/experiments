//! Composition root. The ONE place that sees every layer: it constructs the
//! concrete MemoryRepo, erases it to the usecases.TodoRepo interface, and hands
//! that interface to the http dispatcher inside the accept loop. domain,
//! usecases and http never name a concrete repo — only main does the wiring.
//!
//! Swap MemoryRepo for a SqliteRepo here and nothing else changes.

const std = @import("std");
const memory = @import("memory");
const http = @import("http");
const usecases = @import("usecases");

pub fn main(init: std.process.Init) !void {
    const io = init.io;
    const gpa = init.arena.allocator(); // process-lifetime allocator

    // Port: first CLI arg, else 8080. (e2e passes 0? no — we pass a fixed port.)
    const args = try init.minimal.args.toSlice(gpa);
    const port: u16 = if (args.len > 1)
        std.fmt.parseInt(u16, args[1], 10) catch 8080
    else
        8080;

    // --- Build the concrete repo, expose it as the interface. ---
    var repo_impl = memory.MemoryRepo.init(gpa);
    defer repo_impl.deinit();
    const repo = repo_impl.repo();

    // --- Listen. ---
    const addr = try std.Io.net.IpAddress.parse("127.0.0.1", port);
    var server = try addr.listen(io, .{ .reuse_address = true });
    defer server.deinit(io);

    // Announce readiness on stdout so an e2e harness knows we are up.
    var stdout_buf: [64]u8 = undefined;
    var stdout: std.Io.File.Writer = .init(.stdout(), io, &stdout_buf);
    try stdout.interface.print("listening on {d}\n", .{port});
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
