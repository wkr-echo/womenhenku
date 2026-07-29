import { ThemeProvider } from "@/contexts/ThemeContext";
import { AppProvider } from "@/contexts/AppContext";
import { SidebarView } from "@/components/SidebarView";
import { ContentAreaView } from "@/components/ContentAreaView";
import { ToastContainer } from "@/components/ui/Toast";
import { useKeyboardShortcuts } from "@/hooks/useKeyboard";

function AppLayout() {
  useKeyboardShortcuts();

  return (
    <div className="h-screen flex overflow-hidden">
      <SidebarView />
      <ContentAreaView />
      <ToastContainer />
    </div>
  );
}

export default function App() {
  return (
    <ThemeProvider>
      <AppProvider>
        <AppLayout />
      </AppProvider>
    </ThemeProvider>
  );
}