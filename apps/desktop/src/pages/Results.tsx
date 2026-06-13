import React from "react";
import { useNavigate } from "react-router-dom";
import { open as openShellPath } from "@tauri-apps/plugin-shell";
import { ArrowLeft, Download, FolderOpen, RotateCcw, FileCheck, HelpCircle } from "lucide-react";
import { useAppContext } from "../hooks/useAppContext";
import CodecBreakdown from "../components/CodecBreakdown";

const Results: React.FC = () => {
  const navigate = useNavigate();
  const { compressionResult, setCompressionResult } = useAppContext();

  if (!compressionResult) {
    navigate("/");
    return null;
  }

  const {
    success,
    original_size,
    compressed_size,
    ratio,
    block_count,
    duration_ms,
    codec_usage,
    error,
    is_decompression,
    output_path,
  } = compressionResult;

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
  };

  const handleNewAction = () => {
    setCompressionResult(null);
    navigate("/");
  };

  const handleOpenFile = async () => {
    if (output_path) {
      try {
        await openShellPath(output_path);
      } catch (err) {
        console.error("Failed to open file:", err);
      }
    }
  };

  const handleOpenFolder = async () => {
    if (output_path) {
      try {
        const lastSlash = Math.max(output_path.lastIndexOf("\\"), output_path.lastIndexOf("/"));
        if (lastSlash !== -1) {
          const folder = output_path.substring(0, lastSlash);
          await openShellPath(folder);
        } else {
          await openShellPath(output_path);
        }
      } catch (err) {
        console.error("Failed to open folder:", err);
      }
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-4">
        <button
          onClick={handleNewAction}
          className="qsae-btn-secondary flex items-center gap-2"
        >
          <ArrowLeft className="w-4 h-4" />
          Back
        </button>
        <h2 className="text-xl font-semibold">
          {success
            ? is_decompression
              ? "Decompression Complete"
              : "Compression Complete"
            : is_decompression
            ? "Decompression Failed"
            : "Compression Failed"}
        </h2>
      </div>

      {success ? (
        <>
          {/* Stats Cards */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 animate-fade-in">
            <div className="qsae-card p-6">
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {is_decompression ? "Decompressed Size" : "Original Size"}
              </p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                {formatBytes(original_size)}
              </p>
            </div>
            <div className="qsae-card p-6">
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {is_decompression ? "Compressed Archive" : "Compressed"}
              </p>
              <p className="text-2xl font-bold text-qsae-600">
                {formatBytes(compressed_size)}
              </p>
            </div>
            <div className="qsae-card p-6">
              <p className="text-sm text-gray-500 dark:text-gray-400">Ratio</p>
              <p className="text-2xl font-bold text-green-600">
                {ratio.toFixed(2)}:1
              </p>
              <p className="text-xs text-gray-400">
                {is_decompression
                  ? `${ratio.toFixed(1)}x expansion ratio`
                  : `${((1 - 1 / ratio) * 100).toFixed(1)}% smaller`}
              </p>
            </div>
          </div>

          {/* Details */}
          <div className="qsae-card p-6">
            <h3 className="text-lg font-semibold mb-4">
              {is_decompression ? "Decompression Details" : "Compression Details"}
            </h3>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span className="text-gray-500 dark:text-gray-400">Blocks processed:</span>
                <span className="ml-2 font-medium">{block_count}</span>
              </div>
              {!is_decompression && (
                <>
                  <div>
                    <span className="text-gray-500 dark:text-gray-400">Duration:</span>
                    <span className="ml-2 font-medium">
                      {(duration_ms / 1000).toFixed(1)}s
                    </span>
                  </div>
                  <div>
                    <span className="text-gray-500 dark:text-gray-400">Speed:</span>
                    <span className="ml-2 font-medium">
                      {((original_size / 1024 / 1024) / (duration_ms / 1000)).toFixed(1)} MB/s
                    </span>
                  </div>
                </>
              )}
              <div>
                <span className="text-gray-500 dark:text-gray-400">Efficiency:</span>
                <span className="ml-2 font-medium">
                  {is_decompression
                    ? `${(original_size / compressed_size).toFixed(2)}x expansion`
                    : `${(original_size / compressed_size).toFixed(2)} bytes → 1 byte`}
                </span>
              </div>
            </div>
          </div>

          {/* Codec Usage */}
          {!is_decompression && codec_usage && codec_usage.length > 0 && (
            <div className="qsae-card p-6">
              <h3 className="text-lg font-semibold mb-4">Codec Usage</h3>
              <CodecBreakdown data={codec_usage} />
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-4">
            <button
              onClick={handleOpenFile}
              className="qsae-btn-primary flex items-center gap-2 shadow-md"
            >
              <Download className="w-4 h-4" />
              Open File
            </button>
            <button
              onClick={handleOpenFolder}
              className="qsae-btn-secondary flex items-center gap-2 shadow"
            >
              <FolderOpen className="w-4 h-4" />
              Open Folder
            </button>
            <button
              onClick={handleNewAction}
              className="qsae-btn-secondary flex items-center gap-2 shadow"
            >
              <RotateCcw className="w-4 h-4" />
              {is_decompression ? "Decompress Another" : "Compress Another"}
            </button>
          </div>
        </>
      ) : (
        /* Error State */
        <div className="qsae-card p-8 text-center animate-fade-in">
          <div className="w-16 h-16 bg-red-100 dark:bg-red-900/20 rounded-full flex items-center justify-center mx-auto mb-4">
            <FileCheck className="w-8 h-8 text-red-600" />
          </div>
          <h3 className="text-lg font-semibold text-red-600 mb-2">
            {is_decompression ? "Decompression Failed" : "Compression Failed"}
          </h3>
          <p className="text-gray-600 dark:text-gray-400 mb-4">
            {error || "Unknown error occurred"}
          </p>
          <button onClick={handleNewAction} className="qsae-btn-primary">
            Try Again
          </button>
        </div>
      )}
    </div>
  );
};

export default Results;
