import React from "react";
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler,
} from "chart.js";
import { Line } from "react-chartjs-2";

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, Filler);

interface QuorumSignalProps {
  data: number[];
  switchPoints: number[];
}

const QuorumSignal: React.FC<QuorumSignalProps> = ({ data, switchPoints }) => {
  if (!data || data.length === 0) {
    return (
      <div className="h-48 bg-gray-100 dark:bg-gray-800 rounded-lg flex items-center justify-center">
        <p className="text-gray-400">No quorum data</p>
      </div>
    );
  }

  const threshold = 1.2;
  const labels = data.map((_, i) => i.toString());

  const chartData = {
    labels,
    datasets: [
      {
        label: "Q(i) — Quorum Signal",
        data,
        borderColor: "rgb(14, 165, 233)",
        backgroundColor: "rgba(14, 165, 233, 0.1)",
        fill: true,
        tension: 0.4,
        pointRadius: 0,
        pointHoverRadius: 4,
      },
      {
        label: "Threshold δ",
        data: Array(data.length).fill(threshold),
        borderColor: "rgb(239, 68, 68)",
        borderDash: [5, 5],
        pointRadius: 0,
        fill: false,
      },
    ],
  };

  const options = {
    responsive: true,
    maintainAspectRatio: false,
    interaction: {
      mode: "index" as const,
      intersect: false,
    },
    plugins: {
      legend: {
        position: "top" as const,
      },
      tooltip: {
        callbacks: {
          title: (items: any) => `Block ${items[0].label}`,
          label: (item: any) => {
            if (item.dataset.label === "Threshold δ") return null;
            return `Q(i) = ${item.raw.toFixed(2)}`;
          },
        },
      },
      annotation: {
        annotations: switchPoints.map((point) => ({
          type: "line" as const,
          xMin: point,
          xMax: point,
          borderColor: "rgb(245, 158, 11)",
          borderWidth: 2,
          borderDash: [3, 3],
          label: {
            content: "Switch",
            position: "start" as const,
          },
        })),
      },
    },
    scales: {
      x: {
        title: {
          display: true,
          text: "Block Index",
        },
      },
      y: {
        title: {
          display: true,
          text: "Q(i) Value",
        },
        min: 0,
      },
    },
  };

  return (
    <div className="space-y-4">
      <div className="h-80">
        <Line data={chartData} options={options} />
      </div>

      {switchPoints.length > 0 && (
        <div className="bg-yellow-50 dark:bg-yellow-900/20 p-4 rounded-lg">
          <p className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
            {switchPoints.length} codec switch{switchPoints.length > 1 ? "es" : ""} detected
          </p>
          <p className="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
            At blocks: {switchPoints.join(", ")}
          </p>
        </div>
      )}
    </div>
  );
};

export default QuorumSignal;
