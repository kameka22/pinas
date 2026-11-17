import { describe, expect, it } from "vitest";
import { raidCapacity, recommendedRaid, validRaidCount } from "./raid.js";

describe("pool layouts and usable capacity", () => {
  it("enforces disk counts before confirmation", () => {
    expect(validRaidCount("basic", 2)).toBe(false);
    expect(validRaidCount("jbod", 1)).toBe(false);
    expect(validRaidCount("raid5", 2)).toBe(false);
    expect(validRaidCount("raid10", 5)).toBe(false);
    expect(validRaidCount("raid10", 6)).toBe(true);
    expect(validRaidCount("btrfs-raid10", 5)).toBe(true);
  });
  it("calculates asymmetric disk capacities for mdadm and btrfs", () => {
    expect(raidCapacity("raid1", [2000, 1000])).toBe(1000);
    expect(raidCapacity("raid5", [1000, 2000, 3000])).toBe(2000);
    expect(raidCapacity("raid6", [1000, 2000, 3000, 4000])).toBe(2000);
    expect(raidCapacity("raid10", [1000, 2000, 3000, 4000])).toBe(2000);
    expect(raidCapacity("btrfs-raid1", [1000, 2000, 9000])).toBe(3000);
    expect(raidCapacity("btrfs-raid6", [1000, 2000, 3000, 4000])).toBe(3000);
    expect(raidCapacity("raid1", [])).toBe(0);
  });
  it("recommends redundancy when possible", () => {
    expect([1, 2, 3, 4].map(recommendedRaid)).toEqual([
      "basic",
      "raid1",
      "raid5",
      "raid5",
    ]);
  });
});
