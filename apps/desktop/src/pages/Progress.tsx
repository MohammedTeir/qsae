import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Loader2, Activity } from "lucide-react";
import EntropyHeatmap from "../components/EntropyHeatmap";
import QuorumChart from "../components/QuorumChart";
import CodecBreakdown from "../components/CodecBreakdown";
import { QuorumData } from "../types";
import { useAppContext } from "../hooks/useAppContext";

const Progress: React.FC = () => {
  const navigate = useNavigate();
  const { compressionResult } = useAppContext();
  const [quorumData, setQuorumData] = useState<QuorumData | null>(null);
  const [progress, setProgress] = useState(0);
  const [stage, setStage] = useState("Analyzing...");

  useEffect(() => {
    if (!compressionResult) {
      // If no compression in progress, go back home
      navigate("/");
      return;
    }

    // Simulate progress stages
    const stages = [
      { at: 10, text: "Partitioning blocks..." },
      { at: 25, text: "Scanning entropy..." },
      { at: 40, text: "Computing quorum signals..." },
      { at: 55, text: "Routing to codecs..." },
      { at: 70, text: "Compressing blocks..." },
      { at: 85, text: "Assembling output..." },
      { at: 100, text: "Complete!" },
    ];

    let current = 0;
    const interval = setInterval(() => {
      current += 2;
      setProgress(current);

      const stage_info = stages.find(s => current >= s.at && current < s.at + 15);
      if (stage_info) {
        setStage(stage_info.text);
      }

      if (current >= 100) {
        clearInterval(interval);
      }
    }, 100);

    return () => clearInterval(interval);
  }, [compressionResult, navigate]);

  // Load quorum data if available
  useEffect(() => {
    const loadQuorum = async () => {
      // This would be called with the actual file path
      // For now, show placeholder visualization
    };
    loadQuorum();
  }, []);

  return (
    <div className="space-y-6">
      <div className="qsae-card p-8">
        <div className="flex items-center justify-center mb-6">
          <Loader2 className="w-12 h-12 text-qsae-600 animate-spin" />
        </div>

        <h2 className="text-xl font-semibold text-center mb-2">{stage}</h2>

        {/* Progress Bar */}
        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 mb-4">
          <div
            className="bg-qsae-600 h-3 rounded-full transition-all duration-300"
            style={{ width: `${progress}%` }}
          />
        </div>

        <p className="text-center text-sm text-gray-500 dark:text-gray-400">
          {progress}% complete
        </p>
      </div>

      {/* Live Visualization */}
      {quorumData && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="qsae-card p-6">
            <h3 className="text-sm font-semibold mb-4 flex items-center gap-2">
              <Activity className="w-4 h-4 text-qsae-600" />
              Entropy Heatmap
            </h3>
            <EntropyHeatmap data={quorumData.entropy_profile} />
          </div>

          <div className="qsae-card p-6">
            <h3 className="text-sm font-semibold mb-4">Quorum Signal</h3>
            <QuorumChart data={quorumData.quorum_curve} switchPoints={quorumData.switch_points} />
          </div>
        </div>
      )}

      {/* Placeholder visualization */}
      {!quorumData && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="qsae-card p-6">
            <h3 className="text-sm font-semibold mb-4 flex items-center gap-2">
              <Activity className="w-4 h-4 text-qsae-600" />
              Entropy Heatmap
            </h3>
            <div className="h-32 bg-gray-100 dark:bg-gray-800 rounded-md flex items-center justify-center">
              <p className="text-sm text-gray-400">Analyzing entropy landscape...</p>
            </div>
          </div>

          <div className="qsae-card p-6">
            <h3 className="text-sm font-semibold mb-4">Quorum Signal</h3>
            <div className="h-32 bg-gray-100 dark:bg-gray-800 rounded-md flex items-center justify-center">
              <p className="text-sm text-gray-400">Computing quorum signals...</p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Progress;
