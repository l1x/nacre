import { initTheme } from './modules/theme';
import { initSearch } from './modules/search';
import { initListFeatures } from './modules/list';
import { initInlineEdit } from './modules/edit';
import { initBoardFeatures } from './modules/board';
import { initDragAndDrop } from './modules/dragdrop';
import { initNavigation } from './modules/navigation';
import { initGraph } from './modules/graph';

document.addEventListener('DOMContentLoaded', () => {
    initTheme();
    initSearch();
    initListFeatures();
    initInlineEdit();
    initBoardFeatures();
    initDragAndDrop();
    initNavigation();
    initGraph();

    console.log("Nacre modular frontend initialized");
});