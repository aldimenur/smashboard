import { useCallback, useState } from "react";

export type ToastType = "info" | "success" | "error";

export interface ToastItem {
  id: number;
  message: string;
  type: ToastType;
}

let toastIdCounter = 1;

export function useToast() {
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const showToast = useCallback((message: string, type: ToastType = "info") => {
    const toastId = toastIdCounter;
    toastIdCounter += 1;

    setToasts((prev) => [...prev, { id: toastId, message, type }]);

    window.setTimeout(() => {
      setToasts((prev) => prev.filter((toast) => toast.id !== toastId));
    }, 3000);
  }, []);

  return { toasts, showToast };
}

interface ToastContainerProps {
  toasts: ToastItem[];
}

export function ToastContainer({ toasts }: ToastContainerProps) {
  return (
    <div className="toast-container" aria-live="polite" aria-atomic="true">
      {toasts.map((toast) => (
        <div key={toast.id} className={`toast toast-${toast.type}`}>
          {toast.message}
        </div>
      ))}
    </div>
  );
}
