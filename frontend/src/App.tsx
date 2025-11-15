/**
 * RegelRecht - Beautiful UX Design
 * Tabbed interface with resizable panels, YAML/Blockly toggle, and sync proposals
 */

import { useState, useEffect, useRef } from 'react';
import { ChevronLeft, ChevronRight, X, GripVertical } from 'lucide-react';
import { useEditorStore } from './store/editorStore';
import { initializeBlocklyBlocks } from './utils/blockly';
import LawTabs from './components/LawTabs';
import LeftPanel from './components/ArticleList';
import MiddlePanel from './components/MiddlePanel';
import RightPanel from './components/SyncPanel';

// Law colors - beautiful palette
const LAW_COLORS = [
  '#3B82F6', // Blue
  '#10B981', // Green
  '#F59E0B', // Amber
  '#EF4444', // Red
  '#8B5CF6', // Purple
  '#EC4899', // Pink
];

function App() {
  const { loadAvailableLaws, availableLaws, currentLaw, selectLaw } = useEditorStore();

  const [openLaws, setOpenLaws] = useState<string[]>([]);
  const [activeLawId, setActiveLawId] = useState<string | null>(null);

  // Panel state with resizing
  const [panels, setPanels] = useState({
    left: { visible: true, width: 25 },
    middle: { visible: true, width: 45 },
    right: { visible: true, width: 30 }
  });

  const [isResizing, setIsResizing] = useState<'left' | 'right' | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Initialize
  useEffect(() => {
    initializeBlocklyBlocks();
    loadAvailableLaws();
  }, [loadAvailableLaws]);

  // Auto-open first law
  useEffect(() => {
    if (availableLaws.length > 0 && openLaws.length === 0) {
      const firstLaw = availableLaws[0];
      setOpenLaws([firstLaw.uuid]);
      setActiveLawId(firstLaw.uuid);
      selectLaw(firstLaw.uuid);
    }
  }, [availableLaws, openLaws.length, selectLaw]);

  // Handle law tab click
  const handleLawTabClick = (uuid: string) => {
    setActiveLawId(uuid);
    selectLaw(uuid);
  };

  // Handle new law tab
  const handleAddLaw = (uuid: string) => {
    if (!openLaws.includes(uuid)) {
      setOpenLaws([...openLaws, uuid]);
    }
    setActiveLawId(uuid);
    selectLaw(uuid);
  };

  // Handle close law tab
  const handleCloseLaw = (uuid: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const newOpenLaws = openLaws.filter(id => id !== uuid);
    setOpenLaws(newOpenLaws);

    if (activeLawId === uuid && newOpenLaws.length > 0) {
      const newActive = newOpenLaws[newOpenLaws.length - 1];
      setActiveLawId(newActive);
      selectLaw(newActive);
    }
  };

  // Resize handling
  const handleMouseDown = (divider: 'left' | 'right') => {
    setIsResizing(divider);
  };

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isResizing || !containerRef.current) return;

      const container = containerRef.current;
      const containerWidth = container.offsetWidth;
      const mouseX = e.clientX - container.getBoundingClientRect().left;
      const percentage = (mouseX / containerWidth) * 100;

      if (isResizing === 'left') {
        const newLeftWidth = Math.max(15, Math.min(40, percentage));
        setPanels(prev => ({
          ...prev,
          left: { ...prev.left, width: newLeftWidth },
          middle: { ...prev.middle, width: prev.middle.width + (prev.left.width - newLeftWidth) }
        }));
      } else {
        const newRightWidth = Math.max(20, Math.min(50, 100 - percentage));
        setPanels(prev => ({
          ...prev,
          middle: { ...prev.middle, width: 100 - prev.left.width - newRightWidth },
          right: { ...prev.right, width: newRightWidth }
        }));
      }
    };

    const handleMouseUp = () => {
      setIsResizing(null);
    };

    if (isResizing) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
      return () => {
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
      };
    }
  }, [isResizing]);

  // Get law color
  const getLawColor = (uuid: string) => {
    const index = availableLaws.findIndex(law => law.uuid === uuid);
    return LAW_COLORS[index % LAW_COLORS.length];
  };

  return (
    <div className="flex flex-col h-screen bg-gray-50">
      {/* Law Tabs Header */}
      <LawTabs
        openLaws={openLaws}
        availableLaws={availableLaws}
        activeLawId={activeLawId}
        onTabClick={handleLawTabClick}
        onCloseLaw={handleCloseLaw}
        onAddLaw={handleAddLaw}
        getLawColor={getLawColor}
      />

      {/* Main Content with Resizable Panels */}
      <div ref={containerRef} className="flex flex-1 overflow-hidden relative">
        {/* Left Panel - Article List */}
        {panels.left.visible && (
          <>
            <div
              style={{ width: `${panels.left.width}%` }}
              className="border-r border-gray-300 overflow-hidden"
            >
              <LeftPanel color={activeLawId ? getLawColor(activeLawId) : undefined} />
            </div>

            {/* Left Resize Handle */}
            <div
              className="w-1 bg-gray-300 hover:bg-blue-500 cursor-col-resize flex items-center justify-center group transition-colors"
              onMouseDown={() => handleMouseDown('left')}
            >
              <GripVertical className="w-4 h-4 text-gray-400 group-hover:text-white" />
            </div>
          </>
        )}

        {/* Middle Panel - YAML/Blockly */}
        {panels.middle.visible && (
          <>
            <div
              style={{ width: `${panels.middle.width}%` }}
              className="overflow-hidden"
            >
              <MiddlePanel color={activeLawId ? getLawColor(activeLawId) : undefined} />
            </div>

            {/* Right Resize Handle */}
            <div
              className="w-1 bg-gray-300 hover:bg-blue-500 cursor-col-resize flex items-center justify-center group transition-colors"
              onMouseDown={() => handleMouseDown('right')}
            >
              <GripVertical className="w-4 h-4 text-gray-400 group-hover:text-white" />
            </div>
          </>
        )}

        {/* Right Panel - Sync Proposals */}
        {panels.right.visible && (
          <div
            style={{ width: `${panels.right.width}%` }}
            className="overflow-hidden bg-white border-l border-gray-200"
          >
            <RightPanel />
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
