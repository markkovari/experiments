import { test, expect } from "@playwright/test";
import { staleNotifications } from "@/lib/shared/push";

const tags = (...t: string[]) => t.map((tag) => ({ tag }));

test("closes read messages, keeps unread and newer-than-known ones", () => {
  const shown = tags("m1", "m2", "m3", "m9", "");
  const stale = staleNotifications(shown, { badge: 1, unread: [2], latest: 3 }).map((n) => n.tag);
  expect(stale).toEqual(["m1", "m3", ""]); // m2 unread, m9 newer than this payload knows about; "" = legacy untagged
});

test("everything read closes everything up to latest", () => {
  expect(staleNotifications(tags("m1", "m2"), { badge: 0, unread: [], latest: 2 })).toHaveLength(2);
});
