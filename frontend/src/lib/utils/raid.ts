import type { RaidType } from "#lib/stores/api.js";

export function validRaidCount(type: RaidType, count: number): boolean {
  const min =
    type === "basic" || type === "btrfs-single"
      ? 1
      : ["jbod", "raid0", "raid1", "btrfs-raid0", "btrfs-raid1"].includes(type)
        ? 2
        : ["raid5", "btrfs-raid5"].includes(type)
          ? 3
          : 4;
  return (
    count >= min &&
    (type !== "basic" || count === 1) &&
    (type !== "raid10" || count % 2 === 0)
  );
}

export function recommendedRaid(count: number): RaidType {
  return count < 2 ? "basic" : count === 2 ? "raid1" : "raid5";
}

/** Same capacity estimates as backend RaidType::usable_capacity. */
export function raidCapacity(type: RaidType, sizes: number[]): number {
  if (!sizes.length) return 0;
  const total = sizes.reduce((a, b) => a + b, 0);
  const smallest = Math.min(...sizes),
    largest = Math.max(...sizes),
    n = sizes.length;
  switch (type) {
    case "raid0":
      return smallest * n;
    case "raid1":
      return smallest;
    case "raid5":
      return smallest * Math.max(0, n - 1);
    case "raid6":
      return smallest * Math.max(0, n - 2);
    case "raid10":
      return smallest * Math.floor(n / 2);
    case "btrfs-raid1":
    case "btrfs-raid10":
      return Math.min(Math.floor(total / 2), total - largest);
    case "btrfs-raid5":
      return total - largest;
    case "btrfs-raid6":
      return [...sizes]
        .sort((a, b) => a - b)
        .slice(0, Math.max(0, n - 2))
        .reduce((a, b) => a + b, 0);
    default:
      return total;
  }
}
