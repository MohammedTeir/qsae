import React, { useCallback } from "react";
import { Upload, File } from "lucide-react";

interface DropZoneProps {
  onDrop: (files: FileList) => void;
  onFileSelect: () => void;
}

const DropZone: React.FC<DropZoneProps> = ({ onDrop, onFileSelect }) => {
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    onDrop(e.dataTransfer.files);
  }, [onDrop]);

  return (
    <div
      onDragOver={handleDragOver}
      onDrop={handleDrop}
      className="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-8 text-center hover:border-qsae-500 dark:hover:border-qsae-400 transition-colors cursor-pointer"
      onClick={onFileSelect}
    >
      <div className="flex flex-col items-center gap-3">
        <div className="w-16 h-16 bg-qsae-100 dark:bg-qsae-900/30 rounded-full flex items-center justify-center">
          <Upload className="w-8 h-8 text-qsae-600" />
        </div>
        <div>
          <p className="text-lg font-medium text-gray-700 dark:text-gray-300">
            Drop files here
          </p>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
            or click to browse
          </p>
        </div>
        <div className="flex items-center gap-2 text-xs text-gray-400">
          <File className="w-3 h-3" />
          <span>Supports any file type</span>
        </div>
      </div>
    </div>
  );
};

export default DropZone;
