import React, { useState, useRef, useEffect } from 'react';
import { ChevronLeft, ChevronRight, Play, Check, X, Code, Blocks, Search, Zap, AlertCircle, BookOpen, Link2, GripVertical, Plus } from 'lucide-react';

// Mock data voor verschillende wetten met unieke kleuren
const MOCK_WETTEN = [
  {
    id: 'wet-1',
    title: 'Participatiewet',
    shortTitle: 'PW',
    color: '#3B82F6', // Blue
    articles: [
      {
        id: 'pw-art-1',
        number: '1.1',
        title: 'Begripsbepalingen',
        content: `In deze wet wordt verstaan onder:
a. **algemene bijstand**: bijstand op grond van hoofdstuk 3;
b. **belanghebbende**: de belanghebbende, bedoeld in artikel 1:2, eerste lid, van de Algemene wet bestuursrecht;
c. **inwoner**: degene die ingezetene is in de zin van artikel 1, onderdeel a, van de Wet basisregistratie personen.`
      },
      {
        id: 'pw-art-2',
        number: '2.1', 
        title: 'Recht op algemene bijstand',
        content: `De alleenstaande of het gezin heeft recht op algemene bijstand op grond van dit hoofdstuk, indien hij de middelen tot het voorzien in de noodzakelijke kosten van het bestaan mist en niet in staat is deze kosten uit eigen vermogen te voldoen.`
      }
    ]
  },
  {
    id: 'wet-2',
    title: 'Wet inkomstenbelasting 2001',
    shortTitle: 'Wet IB 2001',
    color: '#10B981', // Green
    articles: [
      {
        id: 'ib-art-1',
        number: '2.1',
        title: 'Belastbaar inkomen uit werk en woning',
        content: `Het belastbare inkomen uit werk en woning is het gezamenlijke bedrag van de belastbare inkomsten uit werk en woning verminderd met de persoonsgebonden aftrek.`
      },
      {
        id: 'ib-art-2',
        number: '2.3',
        title: 'Loon',
        content: `Tot loon behoren al de beloningen die worden genoten uit een dienstbetrekking, waaronder mede begrepen uitkeringen wegens beëindiging van de dienstbetrekking en provisies.`
      }
    ]
  },
  {
    id: 'wet-3',
    title: 'Algemene wet bestuursrecht',
    shortTitle: 'Awb',
    color: '#F59E0B', // Orange
    articles: [
      {
        id: 'awb-art-1',
        number: '1.1',
        title: 'Definities',
        content: `Onder bestuursorgaan wordt verstaan: een orgaan van een rechtspersoon die krachtens publiekrecht is ingesteld, of een ander persoon of college, met enig openbaar gezag bekleed.`
      }
    ]
  }
];

const NRML_DATA = {
  'pw-art-1': `# Artikel 1.1 - Begripsbepalingen
definitions:
  algemene_bijstand:
    type: Uitkering
    basis: hoofdstuk_3
    
  belanghebbende:
    type: Persoon
    referentie: awb.artikel_1_2_lid_1
    
  inwoner:
    type: NatuurlijkPersoon
    constraint: ingezetene == true
    bron: wet_brp.artikel_1_onderdeel_a`,
    
  'pw-art-2': `# Artikel 2.1 - Recht op algemene bijstand
rule: recht_op_bijstand
when:
  - persoon.type == "alleenstaande" OR persoon.type == "gezin"
  - persoon.middelen == false
  - persoon.eigen_vermogen_toereikend == false
then:
  - recht_op_algemene_bijstand: true
  - bijstand_categorie: "hoofdstuk_3"`,
  
  'ib-art-1': `# Artikel 2.1 - Belastbaar inkomen uit werk en woning  
calculation: belastbaar_inkomen_werk_woning
input:
  - belastbare_inkomsten_werk_woning: Bedrag
  - persoonsgebonden_aftrek: Bedrag
formula:
  belastbaar_inkomen = belastbare_inkomsten_werk_woning - persoonsgebonden_aftrek
constraints:
  - belastbaar_inkomen >= 0`,
  
  'ib-art-2': `# Artikel 2.3 - Loon
definition: loon
includes:
  - beloningen_dienstbetrekking
  - uitkeringen_beeindinging
  - provisies
type: Inkomen
category: werk_en_woning`,
  
  'awb-art-1': `# Artikel 1.1 - Definities
definition: bestuursorgaan
conditions:
  - orgaan.rechtspersoon.basis == "publiekrecht"
  - OR persoon.bekleed_met == "openbaar_gezag"
type: Entiteit
juridisch_karakter: publiekrechtelijk`
};

const SEARCH_RESULTS = [
  {
    wet: 'Wet inkomstenbelasting 2001',
    wetId: 'wet-2',
    articleId: 'ib-art-1',
    article: '2.1',
    title: 'Belastbaar inkomen uit werk en woning',
    snippet: 'Het <mark>belastbare inkomen</mark> uit werk en woning...',
    color: '#10B981'
  },
  {
    wet: 'Wet inkomstenbelasting 2001',
    wetId: 'wet-2', 
    articleId: 'ib-art-2',
    article: '2.3',
    title: 'Loon',
    snippet: 'Tot <mark>loon</mark> behoren al de beloningen...',
    color: '#10B981'
  },
  {
    wet: 'Algemene wet bestuursrecht',
    wetId: 'wet-3',
    articleId: 'awb-art-1',
    article: '1.1',
    title: 'Definities',
    snippet: 'Onder <mark>bestuursorgaan</mark> wordt verstaan...',
    color: '#F59E0B'
  }
];

const RegelRechtDemo = () => {
  const [openWetten, setOpenWetten] = useState([MOCK_WETTEN[0]]);
  const [activeWetId, setActiveWetId] = useState('wet-1');
  const [activeArticleId, setActiveArticleId] = useState('pw-art-1');
  const [editingArticle, setEditingArticle] = useState(null);
  const [editedContent, setEditedContent] = useState({});
  const [editedNRML, setEditedNRML] = useState({});
  const [draggedBlock, setDraggedBlock] = useState(null);
  
  const [panels, setPanels] = useState({
    left: { visible: true, width: 33 },
    middle: { visible: true, width: 34 },
    right: { visible: true, width: 33 }
  });
  
  const [middleMode, setMiddleMode] = useState('yaml');
  const [rightTab, setRightTab] = useState('scenarios');
  const [showAtMention, setShowAtMention] = useState(false);
  const [atMentionQuery, setAtMentionQuery] = useState('');
  const [syncProposals, setSyncProposals] = useState([
    {
      id: 'prop-1',
      type: 'nrml_to_text',
      articleId: 'pw-art-2',
      change: 'NRML gewijzigd: "persoon.middelen == false" → "persoon.middelen < 1000"',
      proposal: 'Wijzig tekst naar: "...indien hij de middelen tot een bedrag van € 1.000 mist..."',
      confidence: 0.87
    }
  ]);
  
  const leftScrollRef = useRef(null);
  const middleScrollRef = useRef(null);
  const [isResizing, setIsResizing] = useState(null);
  
  const activeWet = openWetten.find(w => w.id === activeWetId);
  const activeArticle = activeWet?.articles.find(a => a.id === activeArticleId);
  
  const handleWetTabClick = (wetId) => {
    setActiveWetId(wetId);
    const wet = openWetten.find(w => w.id === wetId);
    if (wet?.articles[0]) {
      setActiveArticleId(wet.articles[0].id);
    }
  };
  
  const handleArticleClick = (articleId) => {
    setActiveArticleId(articleId);
    
    if (middleScrollRef.current) {
      const element = middleScrollRef.current.querySelector(`[data-article="${articleId}"]`);
      if (element) {
        element.scrollIntoView({ behavior: 'smooth', block: 'start' });
      }
    }
  };
  
  const handleAtMentionSelect = (result) => {
    const existingWet = openWetten.find(w => w.id === result.wetId);
    
    if (!existingWet) {
      const wetToAdd = MOCK_WETTEN.find(w => w.id === result.wetId);
      if (wetToAdd) {
        setOpenWetten([...openWetten, wetToAdd]);
      }
    }
    
    handleWetTabClick(result.wetId);
    setTimeout(() => {
      handleArticleClick(result.articleId);
    }, 100);
    
    setShowAtMention(false);
    setAtMentionQuery('');
  };
  
  const togglePanel = (panel) => {
    setPanels(prev => ({
      ...prev,
      [panel]: { ...prev[panel], visible: !prev[panel].visible }
    }));
  };
  
  const getActualWidths = () => {
    const visiblePanels = Object.entries(panels).filter(([_, p]) => p.visible);
    const totalWidth = visiblePanels.reduce((sum, [_, p]) => sum + p.width, 0);
    
    return {
      left: panels.left.visible ? (panels.left.width / totalWidth) * 100 : 0,
      middle: panels.middle.visible ? (panels.middle.width / totalWidth) * 100 : 0,
      right: panels.right.visible ? (panels.right.width / totalWidth) * 100 : 0
    };
  };
  
  const actualWidths = getActualWidths();
  
  const acceptProposal = (proposalId) => {
    setSyncProposals(prev => prev.filter(p => p.id !== proposalId));
  };
  
  const rejectProposal = (proposalId) => {
    setSyncProposals(prev => prev.filter(p => p.id !== proposalId));
  };
  
  useEffect(() => {
    const handleMouseMove = (e) => {
      if (!isResizing) return;
      
      const container = document.getElementById('editor-container');
      const rect = container.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const percentage = (x / rect.width) * 100;
      
      if (isResizing === 'left-middle') {
        setPanels(prev => ({
          ...prev,
          left: { ...prev.left, width: Math.max(15, Math.min(70, percentage)) },
          middle: { ...prev.middle, width: Math.max(15, 100 - percentage - prev.right.width) }
        }));
      } else if (isResizing === 'middle-right') {
        setPanels(prev => ({
          ...prev,
          middle: { ...prev.middle, width: Math.max(15, Math.min(70, percentage - prev.left.width)) },
          right: { ...prev.right, width: Math.max(15, 100 - percentage) }
        }));
      }
    };
    
    const handleMouseUp = () => {
      setIsResizing(null);
    };
    
    if (isResizing) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    }
    
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isResizing]);
  
  return (
    <div className="h-screen w-screen flex flex-col bg-gray-100 overflow-hidden">
      {/* Header */}
      <div className="bg-gray-900 text-white px-6 py-3 flex items-center justify-between shadow-lg">
        <div className="flex items-center gap-3">
          <BookOpen size={24} className="text-blue-400" />
          <h1 className="text-xl font-semibold">RegelRecht Editor</h1>
          <span className="text-sm text-gray-400">v2.0 Demo</span>
        </div>
        <div className="flex items-center gap-4 text-sm text-gray-300">
          <span>{activeWet?.shortTitle} - Art. {activeArticle?.number}</span>
          <span className="text-gray-500">|</span>
          <span className="text-green-400">● Live</span>
        </div>
      </div>
      
      {/* Wet Tabs */}
      <div className="bg-gray-200 border-b border-gray-300 px-4 flex items-center gap-1 overflow-x-auto">
        {openWetten.map((wet) => (
          <button
            key={wet.id}
            onClick={() => handleWetTabClick(wet.id)}
            className={`
              px-4 py-2 rounded-t-lg text-sm font-medium transition-all flex items-center gap-2
              ${activeWetId === wet.id 
                ? 'bg-gray-50 text-gray-900 border-t-2' 
                : 'bg-gray-300 text-gray-600 hover:bg-gray-250'
              }
            `}
            style={{
              borderTopColor: activeWetId === wet.id ? wet.color : 'transparent'
            }}
          >
            <div 
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: wet.color }}
            />
            {wet.shortTitle}
            {openWetten.length > 1 && (
              <X 
                size={14} 
                className="hover:text-red-500"
                onClick={(e) => {
                  e.stopPropagation();
                  setOpenWetten(openWetten.filter(w => w.id !== wet.id));
                  if (activeWetId === wet.id && openWetten.length > 1) {
                    setActiveWetId(openWetten[0].id);
                  }
                }}
              />
            )}
          </button>
        ))}
        
        {openWetten.length < 3 && (
          <button 
            className="px-3 py-2 text-sm text-gray-500 hover:text-gray-700"
            onClick={() => {
              const availableWet = MOCK_WETTEN.find(w => !openWetten.find(ow => ow.id === w.id));
              if (availableWet) {
                setOpenWetten([...openWetten, availableWet]);
              }
            }}
          >
            + Wet toevoegen
          </button>
        )}
      </div>
      
      {/* Sync Proposals Banner */}
      {syncProposals.length > 0 && (
        <div className="bg-amber-100 border-b border-amber-300 px-6 py-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Zap size={18} className="text-amber-600" />
              <span className="text-sm font-medium text-amber-900">
                {syncProposals.length} sync voorstel{syncProposals.length > 1 ? 'len' : ''} beschikbaar
              </span>
            </div>
            <button 
              className="text-sm text-amber-700 hover:text-amber-900"
              onClick={() => setRightTab('validation')}
            >
              Bekijk →
            </button>
          </div>
        </div>
      )}
      
      {/* Main editor area */}
      <div id="editor-container" className="flex-1 flex relative overflow-hidden">
        {/* LEFT PANEL */}
        {panels.left.visible ? (
          <div 
            className="border-r-2 border-gray-400 bg-gray-50 flex flex-col relative"
            style={{ width: `${actualWidths.left}%` }}
          >
            {/* Colored accent bar */}
            <div 
              className="absolute top-0 left-0 w-1 h-full"
              style={{ backgroundColor: activeWet?.color }}
            />
            
            <div className="bg-gray-200 px-4 py-2 border-b border-gray-300 flex items-center justify-between">
              <h2 className="font-semibold text-gray-800">Wetgeving (Analoog)</h2>
              <button 
                onClick={() => togglePanel('left')}
                className="p-1 hover:bg-gray-300 rounded"
              >
                <ChevronLeft size={18} />
              </button>
            </div>
            
            <div 
              ref={leftScrollRef}
              className="flex-1 overflow-y-auto p-4 space-y-4 font-serif"
              style={{ fontFamily: 'Georgia, Cambria, "Times New Roman", Times, serif' }}
            >
              {activeWet?.articles.map(article => (
                <div 
                  key={article.id}
                  data-article={article.id}
                  className={`
                    cursor-pointer p-4 rounded-lg transition-all relative
                    ${activeArticleId === article.id 
                      ? 'bg-white shadow-lg ring-2' 
                      : 'bg-gray-100 hover:bg-gray-150 hover:shadow'
                    }
                  `}
                  style={{
                    ringColor: activeArticleId === article.id ? activeWet.color : 'transparent'
                  }}
                  onClick={() => handleArticleClick(article.id)}
                >
                  {activeArticleId === article.id && (
                    <div 
                      className="absolute left-0 top-0 bottom-0 w-1 rounded-l-lg"
                      style={{ backgroundColor: activeWet.color }}
                    />
                  )}
                  
                  <div className="flex items-center gap-2 mb-2">
                    <span 
                      className="text-xs font-bold px-2 py-1 rounded"
                      style={{ 
                        backgroundColor: activeWet.color + '20',
                        color: activeWet.color,
                        fontFamily: 'system-ui'
                      }}
                    >
                      Art. {article.number}
                    </span>
                    <span className="font-bold text-gray-800">{article.title}</span>
                  </div>
                  
                  <div 
                    className="text-sm text-gray-700 whitespace-pre-wrap leading-relaxed"
                    contentEditable={activeArticleId === article.id}
                    suppressContentEditableWarning
                    onBlur={(e) => {
                      setEditedContent({
                        ...editedContent,
                        [article.id]: e.currentTarget.textContent
                      });
                    }}
                    dangerouslySetInnerHTML={{ 
                      __html: (editedContent[article.id] || article.content).replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
                    }}
                  />
                  
                  {/* @-mention demo trigger */}
                  {article.id === 'pw-art-1' && (
                    <div className="mt-3 pt-3 border-t border-gray-200" style={{ fontFamily: 'system-ui' }}>
                      <input
                        type="text"
                        placeholder="Typ @ om te zoeken in wetgeving..."
                        className="w-full px-3 py-2 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-400"
                        value={atMentionQuery}
                        onChange={(e) => {
                          setAtMentionQuery(e.target.value);
                          if (e.target.value.startsWith('@')) {
                            setShowAtMention(true);
                          } else {
                            setShowAtMention(false);
                          }
                        }}
                      />
                      
                      {showAtMention && atMentionQuery.length > 1 && (
                        <div className="mt-2 bg-white border border-gray-300 rounded-lg shadow-xl overflow-hidden z-10">
                          <div className="px-3 py-2 bg-gray-100 border-b border-gray-200 flex items-center gap-2">
                            <Search size={14} className="text-gray-500" />
                            <span className="text-xs font-medium text-gray-600">
                              Zoeken in {SEARCH_RESULTS.length} wetten...
                            </span>
                          </div>
                          {SEARCH_RESULTS
                            .filter(r => 
                              r.wet.toLowerCase().includes(atMentionQuery.slice(1).toLowerCase()) ||
                              r.article.includes(atMentionQuery.slice(1))
                            )
                            .map((result, idx) => (
                            <button
                              key={idx}
                              className="w-full px-3 py-2 text-left hover:bg-blue-50 transition-colors border-b border-gray-100 last:border-b-0"
                              onClick={() => handleAtMentionSelect(result)}
                            >
                              <div className="flex items-center gap-2 mb-1">
                                <div 
                                  className="w-2 h-2 rounded-full"
                                  style={{ backgroundColor: result.color }}
                                />
                                <span className="text-xs font-semibold text-gray-700">
                                  {result.wet}
                                </span>
                                <span className="text-xs text-gray-500">
                                  Art. {result.article}
                                </span>
                              </div>
                              <div className="text-xs text-gray-600 font-medium mb-1">
                                {result.title}
                              </div>
                              <div 
                                className="text-xs text-gray-500"
                                dangerouslySetInnerHTML={{ __html: result.snippet }}
                              />
                            </button>
                          ))}
                        </div>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        ) : (
          <div 
            className="w-8 flex items-center justify-center cursor-pointer hover:bg-gray-300 transition-colors relative"
            style={{ backgroundColor: '#D1D5DB' }}
            onClick={() => togglePanel('left')}
          >
            <div 
              className="absolute left-0 top-0 bottom-0 w-1"
              style={{ backgroundColor: activeWet?.color }}
            />
            <ChevronRight size={18} className="text-gray-700" />
          </div>
        )}
        
        {/* Resizer */}
        {panels.left.visible && panels.middle.visible && (
          <div 
            className="w-1 bg-gray-400 cursor-col-resize hover:bg-blue-500 transition-colors"
            onMouseDown={() => setIsResizing('left-middle')}
          />
        )}
        
        {/* MIDDLE PANEL */}
        {panels.middle.visible ? (
          <div 
            className="bg-gray-100 flex flex-col"
            style={{ width: `${actualWidths.middle}%` }}
          >
            <div className="bg-gray-300 px-4 py-2 border-b border-gray-400 flex items-center justify-between">
              <div className="flex items-center gap-4">
                <h2 className="font-semibold text-gray-800">NRML (Machine-uitvoerbaar)</h2>
                <div className="flex gap-1 bg-gray-200 rounded p-1">
                  <button
                    onClick={() => setMiddleMode('yaml')}
                    className={`px-3 py-1 rounded text-xs flex items-center gap-1 transition-colors ${
                      middleMode === 'yaml' 
                        ? 'bg-gray-700 text-white' 
                        : 'text-gray-600 hover:bg-gray-250'
                    }`}
                  >
                    <Code size={14} /> YAML
                  </button>
                  <button
                    onClick={() => setMiddleMode('blockly')}
                    className={`px-3 py-1 rounded text-xs flex items-center gap-1 transition-colors ${
                      middleMode === 'blockly' 
                        ? 'bg-gray-700 text-white' 
                        : 'text-gray-600 hover:bg-gray-250'
                    }`}
                  >
                    <Blocks size={14} /> Blockly
                  </button>
                </div>
              </div>
              <button 
                onClick={() => togglePanel('middle')}
                className="p-1 hover:bg-gray-400 rounded"
              >
                <ChevronLeft size={18} />
              </button>
            </div>
            
            <div 
              ref={middleScrollRef}
              className="flex-1 overflow-y-auto p-4"
            >
              {middleMode === 'yaml' ? (
                <div className="space-y-4">
                  {activeWet?.articles.map(article => (
                    <div
                      key={article.id}
                      data-article={article.id}
                      className={`rounded-lg transition-all ${
                        activeArticleId === article.id 
                          ? 'ring-2' 
                          : 'opacity-60'
                      }`}
                      style={{
                        ringColor: activeArticleId === article.id ? activeWet.color : 'transparent'
                      }}
                    >
                      <textarea
                        className="w-full bg-gray-800 text-gray-100 p-4 rounded-lg text-sm font-mono leading-relaxed resize-none focus:outline-none focus:ring-2"
                        style={{
                          minHeight: '200px',
                          focusRingColor: activeWet.color
                        }}
                        value={editedNRML[article.id] || NRML_DATA[article.id]}
                        onChange={(e) => {
                          setEditedNRML({
                            ...editedNRML,
                            [article.id]: e.target.value
                          });
                        }}
                        spellCheck={false}
                      />
                    </div>
                  ))}
                </div>
              ) : (
                <div className="bg-white rounded-lg p-6 border-2 border-gray-300 min-h-full">
                  <div className="flex items-center justify-between mb-6">
                    <div>
                      <h3 className="text-lg font-semibold text-gray-800 flex items-center gap-2">
                        <Blocks size={20} />
                        Visual Block Editor
                      </h3>
                      <p className="text-xs text-gray-500 mt-1">
                        Sleep blocks om NRML regels te bouwen
                      </p>
                    </div>
                  </div>
                  
                  {/* Palette van beschikbare blocks */}
                  <div className="mb-6 bg-gray-50 p-4 rounded-lg border border-gray-200">
                    <h4 className="text-xs font-semibold text-gray-600 mb-3 uppercase">Block Palette</h4>
                    <div className="flex flex-wrap gap-2">
                      <div 
                        className="px-3 py-2 rounded shadow-md text-white font-medium text-sm cursor-move hover:opacity-80 transition-opacity flex items-center gap-2"
                        style={{ backgroundColor: activeWet?.color }}
                        draggable
                      >
                        <GripVertical size={14} />
                        WHEN
                      </div>
                      <div 
                        className="px-3 py-2 rounded shadow-md text-white font-medium text-sm cursor-move hover:opacity-80 transition-opacity flex items-center gap-2"
                        style={{ backgroundColor: activeWet?.color }}
                        draggable
                      >
                        <GripVertical size={14} />
                        THEN
                      </div>
                      <div className="bg-green-600 text-white px-3 py-2 rounded shadow-md font-medium text-sm cursor-move hover:opacity-80 transition-opacity flex items-center gap-2" draggable>
                        <GripVertical size={14} />
                        Condition
                      </div>
                      <div className="bg-orange-600 text-white px-3 py-2 rounded shadow-md font-medium text-sm cursor-move hover:opacity-80 transition-opacity flex items-center gap-2" draggable>
                        <GripVertical size={14} />
                        Action
                      </div>
                      <div className="bg-purple-600 text-white px-3 py-2 rounded shadow-md font-medium text-sm cursor-move hover:opacity-80 transition-opacity flex items-center gap-2" draggable>
                        <GripVertical size={14} />
                        Variable
                      </div>
                      <div className="bg-blue-600 text-white px-3 py-2 rounded shadow-md font-medium text-sm cursor-move hover:opacity-80 transition-opacity flex items-center gap-2" draggable>
                        <GripVertical size={14} />
                        Operator
                      </div>
                    </div>
                  </div>
                  
                  {/* Workspace met huidige regel */}
                  <div className="bg-gray-100 p-6 rounded-lg border-2 border-dashed border-gray-300 min-h-[400px]">
                    <div className="mb-4 text-xs font-semibold text-gray-600 uppercase flex items-center justify-between">
                      <span>Workspace - Artikel {activeArticle?.number}</span>
                      <button className="text-blue-600 hover:text-blue-700 normal-case font-normal flex items-center gap-1">
                        <Plus size={14} /> Nieuwe regel
                      </button>
                    </div>
                    
                    {/* Voorbeeld regel voor Artikel 2.1 */}
                    <div className="space-y-3">
                      <div className="bg-white p-4 rounded-lg shadow-sm border border-gray-200">
                        <div className="text-xs font-semibold text-gray-500 mb-3">rule: recht_op_bijstand</div>
                        
                        {/* WHEN block */}
                        <div className="flex items-start gap-2 mb-3">
                          <div 
                            className="px-3 py-2 rounded shadow-md text-white font-medium text-sm flex-shrink-0 cursor-move"
                            style={{ backgroundColor: activeWet?.color }}
                          >
                            WHEN
                          </div>
                          <div className="flex-1 space-y-2 ml-4">
                            <div className="flex items-center gap-2">
                              <div className="bg-green-600 text-white px-3 py-2 rounded shadow-md text-sm flex items-center gap-2 cursor-move">
                                <GripVertical size={12} />
                                persoon.type
                              </div>
                              <div className="bg-blue-600 text-white px-2 py-1 rounded text-xs">
                                ==
                              </div>
                              <input 
                                type="text" 
                                className="px-2 py-1 border border-gray-300 rounded text-sm w-32"
                                defaultValue="alleenstaande"
                              />
                            </div>
                            
                            <div className="flex items-center gap-2">
                              <div className="bg-green-600 text-white px-3 py-2 rounded shadow-md text-sm flex items-center gap-2 cursor-move">
                                <GripVertical size={12} />
                                persoon.middelen
                              </div>
                              <div className="bg-blue-600 text-white px-2 py-1 rounded text-xs">
                                ==
                              </div>
                              <select className="px-2 py-1 border border-gray-300 rounded text-sm">
                                <option>false</option>
                                <option>true</option>
                              </select>
                            </div>
                            
                            <div className="flex items-center gap-2">
                              <div className="bg-green-600 text-white px-3 py-2 rounded shadow-md text-sm flex items-center gap-2 cursor-move">
                                <GripVertical size={12} />
                                persoon.eigen_vermogen_toereikend
                              </div>
                              <div className="bg-blue-600 text-white px-2 py-1 rounded text-xs">
                                ==
                              </div>
                              <select className="px-2 py-1 border border-gray-300 rounded text-sm">
                                <option>false</option>
                                <option>true</option>
                              </select>
                            </div>
                          </div>
                        </div>
                        
                        {/* THEN block */}
                        <div className="flex items-start gap-2 border-t border-gray-200 pt-3">
                          <div 
                            className="px-3 py-2 rounded shadow-md text-white font-medium text-sm flex-shrink-0 cursor-move"
                            style={{ backgroundColor: activeWet?.color }}
                          >
                            THEN
                          </div>
                          <div className="flex-1 space-y-2 ml-4">
                            <div className="flex items-center gap-2">
                              <div className="bg-orange-600 text-white px-3 py-2 rounded shadow-md text-sm flex items-center gap-2 cursor-move">
                                <GripVertical size={12} />
                                recht_op_algemene_bijstand
                              </div>
                              <div className="text-gray-600 text-sm">=</div>
                              <select className="px-2 py-1 border border-gray-300 rounded text-sm">
                                <option>true</option>
                                <option>false</option>
                              </select>
                            </div>
                          </div>
                        </div>
                      </div>
                      
                      {/* Drop zone voor nieuwe regel */}
                      <div className="border-2 border-dashed border-gray-400 rounded-lg p-8 text-center text-gray-400 hover:border-blue-400 hover:text-blue-400 transition-colors cursor-pointer">
                        <Plus size={24} className="mx-auto mb-2" />
                        <p className="text-sm">Sleep blocks hierheen om nieuwe regel te maken</p>
                      </div>
                    </div>
                  </div>
                  
                  <div className="mt-4 text-xs text-gray-500 italic">
                    💡 In de echte implementatie: volledig drag-and-drop met Google Blockly library
                  </div>
                </div>
              )}
            </div>
          </div>
        ) : (
          <div 
            className="bg-gray-300 w-8 flex items-center justify-center cursor-pointer hover:bg-gray-400 transition-colors"
            onClick={() => togglePanel('middle')}
          >
            <ChevronRight size={18} className="text-gray-700" />
          </div>
        )}
        
        {/* Resizer */}
        {panels.middle.visible && panels.right.visible && (
          <div 
            className="w-1 bg-gray-400 cursor-col-resize hover:bg-green-500 transition-colors"
            onMouseDown={() => setIsResizing('middle-right')}
          />
        )}
        
        {/* RIGHT PANEL */}
        {panels.right.visible ? (
          <div 
            className="bg-gray-200 flex flex-col"
            style={{ width: `${actualWidths.right}%` }}
          >
            <div className="bg-gray-400 px-4 py-2 border-b border-gray-500 flex items-center justify-between">
              <div className="flex items-center gap-4">
                <h2 className="font-semibold text-gray-900">Analyse & Testing</h2>
                <div className="flex gap-1 bg-gray-300 rounded p-1">
                  <button
                    onClick={() => setRightTab('scenarios')}
                    className={`px-3 py-1 rounded text-xs transition-colors ${
                      rightTab === 'scenarios' 
                        ? 'bg-gray-700 text-white' 
                        : 'text-gray-700 hover:bg-gray-350'
                    }`}
                  >
                    Scenario's
                  </button>
                  <button
                    onClick={() => setRightTab('validation')}
                    className={`px-3 py-1 rounded text-xs transition-colors flex items-center gap-1 ${
                      rightTab === 'validation' 
                        ? 'bg-gray-700 text-white' 
                        : 'text-gray-700 hover:bg-gray-350'
                    }`}
                  >
                    Validatie
                    {syncProposals.length > 0 && (
                      <span className="bg-amber-500 text-white text-xs rounded-full w-4 h-4 flex items-center justify-center">
                        {syncProposals.length}
                      </span>
                    )}
                  </button>
                </div>
              </div>
              <button 
                onClick={() => togglePanel('right')}
                className="p-1 hover:bg-gray-500 rounded"
              >
                <ChevronRight size={18} />
              </button>
            </div>
            
            <div className="flex-1 overflow-y-auto p-4">
              {rightTab === 'scenarios' ? (
                <div className="space-y-3">
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="font-semibold text-gray-800">Gherkin Scenario's</h3>
                    <button className="bg-gray-700 text-white px-3 py-1 rounded text-sm flex items-center gap-1 hover:bg-gray-800 transition-colors">
                      <Play size={14} /> Run All
                    </button>
                  </div>
                  
                  <div className="bg-white rounded-lg p-3 border-2 border-green-400 shadow">
                    <div className="flex items-start justify-between mb-2">
                      <div className="font-medium text-sm text-gray-800">
                        Alleenstaande zonder middelen krijgt bijstand
                      </div>
                      <Check size={18} className="text-green-600 flex-shrink-0" />
                    </div>
                    <pre className="text-xs bg-gray-50 p-2 rounded overflow-x-auto text-gray-700 whitespace-pre-wrap">
{`Scenario: Alleenstaande zonder middelen
  Gegeven een persoon van type "alleenstaande"
  En de persoon heeft geen middelen
  En het eigen vermogen is niet toereikend
  Dan heeft de persoon recht op algemene bijstand`}
                    </pre>
                    <div className="mt-2 text-xs text-green-600 bg-green-50 p-2 rounded">
                      ✓ Passed in 0.03s
                    </div>
                  </div>
                  
                  <div className="bg-white rounded-lg p-3 border-2 border-green-400 shadow">
                    <div className="flex items-start justify-between mb-2">
                      <div className="font-medium text-sm text-gray-800">
                        Gezin met toereikend vermogen geen bijstand
                      </div>
                      <Check size={18} className="text-green-600 flex-shrink-0" />
                    </div>
                    <pre className="text-xs bg-gray-50 p-2 rounded overflow-x-auto text-gray-700 whitespace-pre-wrap">
{`Scenario: Gezin met vermogen
  Gegeven een persoon van type "gezin"
  En het eigen vermogen is toereikend
  Dan heeft de persoon geen recht op bijstand`}
                    </pre>
                    <div className="mt-2 text-xs text-green-600 bg-green-50 p-2 rounded">
                      ✓ Passed in 0.02s
                    </div>
                  </div>
                </div>
              ) : (
                <div className="space-y-3">
                  <h3 className="font-semibold text-gray-800 mb-4">Validatie & Sync</h3>
                  
                  {/* Sync Proposals */}
                  {syncProposals.length > 0 && (
                    <div className="mb-4">
                      <div className="flex items-center gap-2 mb-2">
                        <Zap size={16} className="text-amber-600" />
                        <span className="text-sm font-semibold text-gray-800">Sync Voorstellen</span>
                      </div>
                      
                      {syncProposals.map(proposal => (
                        <div key={proposal.id} className="bg-amber-50 rounded-lg p-3 border-2 border-amber-300 mb-2">
                          <div className="flex items-start gap-2 mb-2">
                            <AlertCircle size={16} className="text-amber-600 flex-shrink-0 mt-0.5" />
                            <div className="flex-1">
                              <div className="text-sm font-medium text-gray-800 mb-1">
                                {proposal.type === 'nrml_to_text' ? 'NRML → Tekst' : 'Tekst → NRML'}
                              </div>
                              <div className="text-xs text-gray-600 mb-2">
                                {proposal.change}
                              </div>
                              <div className="text-xs bg-white p-2 rounded border border-amber-200 mb-2">
                                <strong>Voorstel:</strong> {proposal.proposal}
                              </div>
                              <div className="flex items-center gap-2 text-xs text-gray-600">
                                <span>Confidence: {(proposal.confidence * 100).toFixed(0)}%</span>
                                <div className="flex-1 bg-gray-200 rounded-full h-1.5">
                                  <div 
                                    className="bg-amber-500 h-1.5 rounded-full"
                                    style={{ width: `${proposal.confidence * 100}%` }}
                                  />
                                </div>
                              </div>
                            </div>
                          </div>
                          <div className="flex gap-2 mt-2">
                            <button 
                              onClick={() => acceptProposal(proposal.id)}
                              className="flex-1 bg-green-600 text-white px-3 py-1.5 rounded text-xs font-medium hover:bg-green-700 transition-colors flex items-center justify-center gap-1"
                            >
                              <Check size={14} /> Accepteren
                            </button>
                            <button 
                              onClick={() => rejectProposal(proposal.id)}
                              className="flex-1 bg-gray-500 text-white px-3 py-1.5 rounded text-xs font-medium hover:bg-gray-600 transition-colors flex items-center justify-center gap-1"
                            >
                              <X size={14} /> Afwijzen
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                  
                  {/* Validation results */}
                  <div className="bg-white rounded-lg p-3 border-2 border-green-400 shadow">
                    <div className="flex items-center gap-2 mb-2">
                      <Check size={18} className="text-green-600" />
                      <span className="font-medium text-sm">NRML syntaxis correct</span>
                    </div>
                    <p className="text-xs text-gray-600">
                      Alle YAML definities zijn geldig
                    </p>
                  </div>
                  
                  <div className="bg-white rounded-lg p-3 border-2 border-green-400 shadow">
                    <div className="flex items-center gap-2 mb-2">
                      <Check size={18} className="text-green-600" />
                      <span className="font-medium text-sm">Coverage: 100%</span>
                    </div>
                    <p className="text-xs text-gray-600 mb-2">
                      Alle {activeWet?.articles.length} artikelen vertaald naar NRML
                    </p>
                    <div className="flex gap-1">
                      {activeWet?.articles.map(article => (
                        <div 
                          key={article.id}
                          className="flex-1 h-2 rounded"
                          style={{ backgroundColor: activeWet.color }}
                          title={`Art. ${article.number}`}
                        />
                      ))}
                    </div>
                  </div>
                  
                  <div className="bg-white rounded-lg p-3 border-2 border-blue-400 shadow">
                    <div className="flex items-center gap-2 mb-2">
                      <Link2 size={18} className="text-blue-600" />
                      <span className="font-medium text-sm">Cross-referenties</span>
                    </div>
                    <p className="text-xs text-gray-600 mb-2">
                      {openWetten.length - 1} externe referentie{openWetten.length - 1 !== 1 ? 's' : ''}
                    </p>
                    {openWetten.slice(1).map(wet => (
                      <div key={wet.id} className="flex items-center gap-2 text-xs text-gray-700 mt-1">
                        <div 
                          className="w-2 h-2 rounded-full"
                          style={{ backgroundColor: wet.color }}
                        />
                        <span>{wet.shortTitle}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        ) : (
          <div 
            className="bg-gray-400 w-8 flex items-center justify-center cursor-pointer hover:bg-gray-500 transition-colors"
            onClick={() => togglePanel('right')}
          >
            <ChevronLeft size={18} className="text-gray-900" />
          </div>
        )}
      </div>
      
      {/* Status bar */}
      <div className="bg-gray-900 text-gray-300 px-6 py-2 flex items-center justify-between text-xs border-t border-gray-700">
        <div className="flex items-center gap-4">
          <span>Mode: {middleMode.toUpperCase()}</span>
          <span className="text-gray-600">|</span>
          <span>{openWetten.length} wet{openWetten.length > 1 ? 'ten' : ''} open</span>
          <span className="text-gray-600">|</span>
          <span className="text-green-400">Alle scenario's passed</span>
          {syncProposals.length > 0 && (
            <>
              <span className="text-gray-600">|</span>
              <span className="text-amber-400">{syncProposals.length} sync voorstel</span>
            </>
          )}
        </div>
        <div className="flex items-center gap-3">
          <span className="text-gray-500">Laatste sync: zojuist</span>
          <span className="text-gray-600">|</span>
          <span>RegelRecht v2.0.0</span>
        </div>
      </div>
      
      <style>{`
        .bg-gray-150 { background-color: #f7f7f7; }
        .bg-gray-250 { background-color: #e5e5e5; }
        .bg-gray-350 { background-color: #c5c5c5; }
        
        mark {
          background-color: #fef08a;
          padding: 0 2px;
          border-radius: 2px;
        }
        
        /* Improved scrollbar for serif text */
        .font-serif::-webkit-scrollbar {
          width: 10px;
        }
        
        .font-serif::-webkit-scrollbar-track {
          background: #f1f1f1;
        }
        
        .font-serif::-webkit-scrollbar-thumb {
          background: #888;
          border-radius: 5px;
        }
        
        .font-serif::-webkit-scrollbar-thumb:hover {
          background: #555;
        }
      `}</style>
    </div>
  );
};

export default RegelRechtDemo;
