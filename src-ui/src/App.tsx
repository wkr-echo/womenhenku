import { ThemeProvider } from "@/contexts/ThemeContext";
import { AppProvider } from "@/contexts/AppContext";
import { Sidebar } from "@/components/Sidebar";
import { ContentArea } from "@/components/ContentArea";
import { ToastContainer } from "@/components/ui/Toast";
import { useKeyboardShortcuts } from "@/hooks/useKeyboard";

function AppLayout() {
  useKeyboardShortcuts();

  return (
    <div className="h-screen flex overflow-hidden">
      <Sidebar />
      <ContentArea />
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