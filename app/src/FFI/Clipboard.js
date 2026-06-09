// Best-effort clipboard write. The Clipboard API is async and may be
// unavailable (insecure context); failures are swallowed so the UI
// never crashes on a copy.
export const copyToClipboard = (text) => () => {
  try {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text);
    }
  } catch (_) {
    /* ignore */
  }
};
