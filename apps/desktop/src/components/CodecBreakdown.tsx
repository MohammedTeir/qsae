import React from "react";

interface CodecBreakdownProps {
  data: [string, number, number][];
}

const CodecBreakdown: React.FC<CodecBreakdownProps> = ({ data }) => {
  if (!data || data.length === 0) {
    return (
      <div className="text-center py-8 text-gray-400">
        No codec usage data available
      </div>
    );
  }

  // Color map for codecs
  const codecColors: Record<string, string> = {
    "RLE": "#22c55e",
    "LZ4": "#3b82f6",
    "LZ77": "#6366f1",
    "Huffman": "#8b5cf6",
    "ANS": "#a855f7",
    "BWT": "#ec4899",
    "Delta": "#f43f5e",
    "Skip": "#6b7280",
    "DEFLATE": "#9ca3af",
  };

  const getColor = (name: string): string => {
    return codecColors[name] || "#6b7280";
  };

  const total = data.reduce((sum, [, count]) => sum + count, 0);

  return (
    <div className="space-y-4">
      {/* Bar chart */}
      <div className="space-y-3">
        {data.map(([name, count, percentage]) => {
          const barWidth = Math.max(percentage, 1);
          const color = getColor(name);

          return (
            <div key={name} className="flex items-center gap-3">
              <div className="w-20 text-sm font-medium text-gray-700 dark:text-gray-300">
                {name}
              </div>
              <div className="flex-1 h-6 bg-gray-100 dark:bg-gray-700 rounded-full overflow-hidden">
                <div
                  className="h-full rounded-full transition-all duration-500"
                  style={{
                    width: `${barWidth}%`,
                    backgroundColor: color,
                    minWidth: "4px",
                  }}
                />
              </div>
              <div className="w-24 text-right text-sm">
                <span className="font-medium">{count}</span>
                <span className="text-gray-400 ml-1">({percentage.toFixed(1)}%)</span>
              </div>
            </div>
          );
        })}
      </div>

      {/* Summary */}
      <div className="pt-4 border-t border-gray-200 dark:border-gray-700">
        <div className="flex justify-between text-sm text-gray-500">
          <span>Total blocks: {total}</span>
          <span>Codecs used: {data.length}</span>
        </div>
      </div>

      {/* Pie chart representation (simple CSS) */}
      <div className="flex justify-center pt-4">
        <div className="flex gap-2 flex-wrap">
          {data.map(([name, , percentage]) => (
            <div key={name} className="flex items-center gap-1 text-xs">
              <div
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: getColor(name) }}
              />
              <span className="text-gray-600 dark:text-gray-400">{name}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default CodecBreakdown;
