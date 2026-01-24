
import { AppShell } from './app/layout/AppShell';
import { CommandPalette } from './app/layout/CommandPalette';
import { useKeyboardShortcuts } from './commands';
import { useAppStore } from './state/store';

// Page imports
import { ProjectsPage } from './app/pages/ProjectsPage';
import { VisionPage } from './app/pages/VisionPage';
import { AudioPage } from './app/pages/AudioPage';
import { AgentsPage } from './app/pages/AgentsPage';
import { TrainingPage } from './app/pages/TrainingPage';
import { RunsPage } from './app/pages/RunsPage';
import { SettingsPage } from './app/pages/SettingsPage';

/**
 * Main Application Component
 * 
 * Renders the app shell with dynamic page content based on active module.
 * Implements global keyboard shortcuts and command palette.
 */
function App() {
  const { activeModule, isCommandPaletteOpen } = useAppStore();

  // Initialize keyboard shortcuts
  useKeyboardShortcuts();

  // Dynamic page rendering based on active module
  const renderPage = () => {
    switch (activeModule) {
      case 'projects':
        return <ProjectsPage />;
      case 'vision':
        return <VisionPage />;
      case 'audio':
        return <AudioPage />;
      case 'agents':
        return <AgentsPage />;
      case 'training':
        return <TrainingPage />;
      case 'runs':
        return <RunsPage />;
      case 'settings':
        return <SettingsPage />;
      default:
        return <ProjectsPage />;
    }
  };

  return (
    <>
      <AppShell>
        {renderPage()}
      </AppShell>

      {/* Command Palette Overlay */}
      {isCommandPaletteOpen && <CommandPalette />}
    </>
  );
}

export default App;
