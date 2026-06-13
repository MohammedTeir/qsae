import React from "react";

interface CodecMapProps {
  assignments: number[];
  names: string[];
  counts: number[];
}

const CODEC_COLORS: Record<number, string> = {
  0x00: "#6b7280", // Skip
  0x01: "#22c55e", // RLE
  0x02: "#3b82f6", // LZ4
  0x03: "#6366f1", // LZ77
  0x04: "#8b5cf6", // Huffman
  0x05: "#a855f7", // ANS
  0x06: "#ec4899", // BWT
  0x07: "#f43f5e", // Delta
  0x08: "#9ca3af", // DEFLATE
};

const CODEC_NAMES: Record<number, string> = {
  0x00: "Skip",
  0x01: "RLE",
  0x02: "LZ4",
  0x03: "LZ77",
  0x04: "Huffman",
  0x05: "ANS",
  0x06: "BWT",
  0x07: "Delta",
  0x08: "DEFLATE",
};

const CodecMap: React.FC<CodecMapProps> = ({ assignments, names, counts }) => {
  if (!assignments || assignments.length === 0) {
    return (
      <div className="h-48 bg-gray-100 dark:bg-gray-800 rounded-lg flex items-center justify-center">
        <p className="text-gray-400">No codec assignments</p>
      </div>
    );
  }

  const maxCols = Math.min(60, Math.ceil(Math.sqrt(assignments.length * 2)));
  const blockSize = Math.max(6, Math.min(20, Math.floor(600 / maxCols)));

  // Build legend
  const uniqueCodecs = [...new Set(assignments)].sort();
  const total = assignments.length;

  return (
    <div className="space-y-4">
      {/* Codec Map */}
      <div className="flex flex-wrap gap-1">
        {assignments.map((codec, i) => (
          <div
            key={i}
            className="rounded-sm transition-all hover:scale-125"
            style={{
              width: blockSize,
              height: blockSize,
              backgroundColor: CODEC_COLORS[codec] || "#6b7280",
            }}
            title={`Block ${i}: ${CODEC_NAMES[codec] || "Unknown"}`}
          />
        ))}
      </div>

      {/* Legend */}
      <div className="flex flex-wrap gap-3">
        {uniqueCodecs.map((codec) => {
          const count = assignments.filter((a) => a === codec).length;
          const pct = ((count / total) * 100).toFixed(1);
          return (
            <div key={codec} className="flex items-center gap-2 text-sm">
              <div
                className="w-4 h-4 rounded"
                style={{ backgroundColor: CODEC_COLORS[codec] || "#6b7280" }}
              />
              <span className="font-medium">{CODEC_NAMES[codec] || "Unknown"}</span>
              <span className="text-gray-500">
                {count} ({pct}%)
              </span>
            </div>
          );
        })}
      </div>

      {/* Bar Chart */}
      <div className="space-y-2 mt-4">
        {uniqueCodecs.map((codec) => {
          const count = assignments.filter((a) => a === codec).length;
          const pct = (count / total) * 100;
          return (
            <div key={codec} className="flex items-center gap-3">
              <div className="w-20 text-sm font-medium">{CODEC_NAMES[codec]}</div>
              <div className="flex-1 h-6 bg-gray-100 dark:bg-gray-800 rounded-full overflow-hidden">
                <div
                  className="h-full rounded-full transition-all"
                  style={{
                    width: `${Math.max(pct, 1)}%`,
                    backgroundColor: CODEC_COLORS[codec] || "#6b7280",
                  }}
                />
              </div>
              <div className="w-20 text-right text-sm">
                {count} ({pct.toFixed(1)}%)
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default CodecMap;
