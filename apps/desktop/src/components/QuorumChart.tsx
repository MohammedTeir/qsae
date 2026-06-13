import React from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
} from "recharts";

interface QuorumChartProps {
  data: number[];
  switchPoints?: number[];
}

const QuorumChart: React.FC<QuorumChartProps> = ({ data, switchPoints = [] }) => {
  if (!data || data.length === 0) {
    return (
      <div className="h-48 bg-gray-100 dark:bg-gray-800 rounded-md flex items-center justify-center">
        <p className="text-sm text-gray-400">No quorum data available</p>
      </div>
    );
  }

  // Prepare chart data
  const chartData = data.map((value, index) => ({
    index,
    quorum: value,
    isSwitch: switchPoints.includes(index),
  }));

  const threshold = 1.2; // Default delta threshold

  return (
    <div className="h-64">
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={chartData} margin={{ top: 5, right: 20, bottom: 5, left: 0 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#374151" opacity={0.3} />
          <XAxis 
            dataKey="index" 
            tick={{ fontSize: 12 }}
            label={{ value: "Block Index", position: "insideBottom", offset: -5, fontSize: 12 }}
          />
          <YAxis 
            tick={{ fontSize: 12 }}
            label={{ value: "Q(i)", angle: -90, position: "insideLeft", fontSize: 12 }}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: "#1f2937",
              border: "1px solid #374151",
              borderRadius: "6px",
              fontSize: "12px",
            }}
            formatter={(value: number) => [`Q(i) = ${value.toFixed(2)}`, "Quorum Signal"]}
          />
          <ReferenceLine 
            y={threshold} 
            stroke="#ef4444" 
            strokeDasharray="5 5"
            label={{ value: "Threshold δ", position: "right", fontSize: 10, fill: "#ef4444" }}
          />
          <Line
            type="monotone"
            dataKey="quorum"
            stroke="#0ea5e9"
            strokeWidth={2}
            dot={false}
            activeDot={{ r: 4, fill: "#0ea5e9" }}
          />
          {/* Switch points */}
          {switchPoints.map((point) => (
            <ReferenceLine
              key={point}
              x={point}
              stroke="#f59e0b"
              strokeDasharray="3 3"
              strokeWidth={1}
            />
          ))}
        </LineChart>
      </ResponsiveContainer>

      {switchPoints.length > 0 && (
        <div className="mt-2 text-xs text-gray-500">
          Switch points: {switchPoints.length} detected
        </div>
      )}
    </div>
  );
};

export default QuorumChart;
