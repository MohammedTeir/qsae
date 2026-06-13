import React, { useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { FileArchive, Settings, ArrowRight, Zap, FolderOpen, Loader2 } from "lucide-react";
import DropZone from "../components/DropZone";
import ModeSelector from "../components/ModeSelector";
import ParamSliders from "../components/ParamSliders";
import { CompressionParams, CompressionResult } from "../types";
import { useAppContext } from "../hooks/useAppContext";

const Home: React.FC = () => {
  const navigate = useNavigate();
  const { setCompressionResult } = useAppContext();

  const [activeTab, setActiveTab] = useState<"compress" | "decompress">("compress");

  // Compress States
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [outputPath, setOutputPath] = useState<string>("");
  const [params, setParams] = useState<CompressionParams>({
    lambda: 0.5,
    delta: 1.2,
    block_size: 65536,
    use_quorum: true,
  });
  const [isCompressing, setIsCompressing] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);

  // Decompress States
  const [selectedDecompressFile, setSelectedDecompressFile] = useState<string | null>(null);
  const [decompressOutputPath, setDecompressOutputPath] = useState<string>("");
  const [isDecompressing, setIsDecompressing] = useState(false);

  const handleFileSelect = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [{
        name: "All Files",
        extensions: ["*"],
      }],
    });
    if (selected && typeof selected === "string") {
      setSelectedFile(selected);
      const base = selected.replace(/\.[^/.]+$/, "");
      setOutputPath(`${base}.qsae`);
    }
  }, []);

  const handleDecompressFileSelect = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [{
        name: "QSAE Archive",
        extensions: ["qsae"],
      }],
    });
    if (selected && typeof selected === "string") {
      setSelectedDecompressFile(selected);
      const base = selected.replace(/\.qsae$/i, "");
      setDecompressOutputPath(base);
    }
  }, []);
  const handleSaveFileSelect = useCallback(async () => {
    const selected = await save({
      title: "Select Output QSAE Archive",
      defaultPath: outputPath || undefined,
      filters: [{
        name: "QSAE Archive",
        extensions: ["qsae"]
      }]
    });
    if (selected) {
      setOutputPath(selected);
    }
  }, [outputPath]);

  const handleDecompressSaveFileSelect = useCallback(async () => {
    const selected = await save({
      title: "Select Decompressed Output Destination",
      defaultPath: decompressOutputPath || undefined,
    });
    if (selected) {
      setDecompressOutputPath(selected);
    }
  }, [decompressOutputPath]);

  const handleDrop = useCallback((files: FileList) => {
    if (files.length > 0) {
      const file = files[0];
      setSelectedFile(file.name);
    }
  }, []);

  const handleDecompressDrop = useCallback((files: FileList) => {
    if (files.length > 0) {
      const file = files[0];
      setSelectedDecompressFile(file.name);
    }
  }, []);

  const handleCompress = async () => {
    if (!selectedFile || !outputPath) return;

    setIsCompressing(true);
    navigate("/progress");

    try {
      const result: CompressionResult = await invoke("compress_file", {
        request: {
          input_path: selectedFile,
          output_path: outputPath,
          lambda: params.lambda,
          delta: params.delta,
          block_size: params.block_size,
          use_quorum: params.use_quorum,
        },
      });

      setCompressionResult({
        ...result,
        input_path: selectedFile,
        output_path: outputPath,
        is_decompression: false,
      });
      navigate("/results");
    } catch (error) {
      console.error("Compression failed:", error);
      setCompressionResult({
        success: false,
        original_size: 0,
        compressed_size: 0,
        ratio: 0,
        block_count: 0,
        duration_ms: 0,
        codec_usage: [],
        error: String(error),
        input_path: selectedFile,
        output_path: outputPath,
        is_decompression: false,
      });
      navigate("/results");
    } finally {
      setIsCompressing(false);
    }
  };

  const handleDecompress = async () => {
    if (!selectedDecompressFile || !decompressOutputPath) return;

    setIsDecompressing(true);
    try {
      // First try to inspect file to fetch stats
      const inspectResult: any = await invoke("inspect_file", { path: selectedDecompressFile });
      
      const success = await invoke("decompress_file", {
        inputPath: selectedDecompressFile,
        outputPath: decompressOutputPath,
      });

      if (success) {
        setCompressionResult({
          success: true,
          original_size: inspectResult.original_size,
          compressed_size: inspectResult.compressed_size,
          ratio: inspectResult.ratio,
          block_count: inspectResult.block_count,
          duration_ms: 0,
          codec_usage: inspectResult.codec_breakdown,
          input_path: selectedDecompressFile,
          output_path: decompressOutputPath,
          is_decompression: true,
        });
        navigate("/results");
      } else {
        throw new Error("Decompression process failed to complete successfully.");
      }
    } catch (error) {
      console.error("Decompression failed:", error);
      setCompressionResult({
        success: false,
        original_size: 0,
        compressed_size: 0,
        ratio: 0,
        block_count: 0,
        duration_ms: 0,
        codec_usage: [],
        error: String(error),
        input_path: selectedDecompressFile,
        output_path: decompressOutputPath,
        is_decompression: true,
      });
      navigate("/results");
    } finally {
      setIsDecompressing(false);
    }
  };

  const handleModeChange = (mode: "fast" | "balanced" | "max") => {
    switch (mode) {
      case "fast":
        setParams({ ...params, lambda: 0.8, delta: 2.0, block_size: 131072 });
        break;
      case "balanced":
        setParams({ ...params, lambda: 0.5, delta: 1.2, block_size: 65536 });
        break;
      case "max":
        setParams({ ...params, lambda: 0.3, delta: 0.8, block_size: 32768 });
        break;
    }
  };

  return (
    <div className="space-y-6">
      {/* Premium Tab Switcher */}
      <div className="flex bg-gray-200 dark:bg-gray-800 p-1 rounded-lg w-fit mx-auto shadow-inner">
        <button
          onClick={() => setActiveTab("compress")}
          className={`flex items-center gap-2 px-6 py-2.5 rounded-md font-medium text-sm transition-all duration-200 ${
            activeTab === "compress"
              ? "bg-white dark:bg-gray-700 shadow text-qsae-600 dark:text-white"
              : "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          }`}
        >
          <FileArchive className="w-4 h-4" />
          Compress
        </button>
        <button
          onClick={() => setActiveTab("decompress")}
          className={`flex items-center gap-2 px-6 py-2.5 rounded-md font-medium text-sm transition-all duration-200 ${
            activeTab === "decompress"
              ? "bg-white dark:bg-gray-700 shadow text-qsae-600 dark:text-white"
              : "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          }`}
        >
          <FolderOpen className="w-4 h-4" />
          Decompress
        </button>
      </div>

      {activeTab === "compress" ? (
        <>
          {/* Compress Form */}
          <div className="qsae-card p-8 transition-all duration-300">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <FileArchive className="w-5 h-5 text-qsae-600" />
              Select File to Compress
            </h2>

            <DropZone onDrop={handleDrop} onFileSelect={handleFileSelect} />

            {selectedFile && (
              <div className="mt-4 space-y-4 p-4 bg-qsae-50 dark:bg-qsae-900/10 rounded-md border border-qsae-100 dark:border-qsae-900/30 animate-fade-in">
                <div>
                  <label className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    Selected Input File
                  </label>
                  <p className="text-sm font-medium text-gray-800 dark:text-gray-200 mt-0.5">{selectedFile}</p>
                </div>
                
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    Output File Location
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={outputPath}
                      onChange={(e) => setOutputPath(e.target.value)}
                      className="flex-1 px-3 py-1.5 text-sm bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded shadow-sm focus:outline-none focus:ring-1 focus:ring-qsae-500 focus:border-qsae-500 text-gray-800 dark:text-gray-200"
                    />
                    <button
                      onClick={handleSaveFileSelect}
                      className="px-3 py-1.5 text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors shadow-sm"
                    >
                      Browse...
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>

          <div className="qsae-card p-6">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Zap className="w-5 h-5 text-qsae-600" />
              Compression Mode
            </h2>
            <ModeSelector onChange={handleModeChange} />
          </div>

          <div className="qsae-card p-6">
            <button
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-qsae-600 w-full"
            >
              <Settings className="w-4 h-4" />
              Advanced Settings
              <span className="ml-auto">{showAdvanced ? "▾" : "▸"}</span>
            </button>

            {showAdvanced && (
              <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700 animate-slide-down">
                <ParamSliders params={params} onChange={setParams} />
              </div>
            )}
          </div>

          <div className="flex justify-end">
            <button
              onClick={handleCompress}
              disabled={!selectedFile || isCompressing}
              className="qsae-btn-primary flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed shadow-md"
            >
              {isCompressing ? (
                <>
                  <Loader2 className="w-5 h-5 animate-spin" />
                  Compressing...
                </>
              ) : (
                <>
                  <ArrowRight className="w-5 h-5" />
                  Compress File
                </>
              )}
            </button>
          </div>
        </>
      ) : (
        <>
          {/* Decompress Form */}
          <div className="qsae-card p-8 transition-all duration-300">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <FolderOpen className="w-5 h-5 text-qsae-600" />
              Select File to Decompress
            </h2>

            <DropZone onDrop={handleDecompressDrop} onFileSelect={handleDecompressFileSelect} />

            {selectedDecompressFile && (
              <div className="mt-4 space-y-4 p-4 bg-qsae-50 dark:bg-qsae-900/10 rounded-md border border-qsae-100 dark:border-qsae-900/30 animate-fade-in">
                <div>
                  <label className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    Selected Compressed Archive
                  </label>
                  <p className="text-sm font-medium text-gray-800 dark:text-gray-200 mt-0.5">{selectedDecompressFile}</p>
                </div>

                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    Decompressed Output Destination
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={decompressOutputPath}
                      onChange={(e) => setDecompressOutputPath(e.target.value)}
                      className="flex-1 px-3 py-1.5 text-sm bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded shadow-sm focus:outline-none focus:ring-1 focus:ring-qsae-500 focus:border-qsae-500 text-gray-800 dark:text-gray-200"
                    />
                    <button
                      onClick={handleDecompressSaveFileSelect}
                      className="px-3 py-1.5 text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors shadow-sm"
                    >
                      Browse...
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>

          <div className="flex justify-end">
            <button
              onClick={handleDecompress}
              disabled={!selectedDecompressFile || isDecompressing}
              className="qsae-btn-primary flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed shadow-md"
            >
              {isDecompressing ? (
                <>
                  <Loader2 className="w-5 h-5 animate-spin" />
                  Decompressing...
                </>
              ) : (
                <>
                  <ArrowRight className="w-5 h-5" />
                  Decompress File
                </>
              )}
            </button>
          </div>
        </>
      )}
    </div>
  );
};

export default Home;
