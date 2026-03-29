import { useState, useEffect, useRef } from 'react';
import { aiModelApi, ModelStatus } from '../../api/ai_model';
import { Button } from '../ui/Button';

export function AISearchSettings() {
  const [status, setStatus] = useState<ModelStatus | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [downloadedMB, setDownloadedMB] = useState(0);
  const [totalMB, setTotalMB] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [serverRunning, setServerRunning] = useState(false);
  const [starting, setStarting] = useState(false);
  const eventsReceived = useRef(false);

  useEffect(() => {
    aiModelApi.status().then(setStatus);
    aiModelApi.health().then(setServerRunning);
  }, []);

  // Set up Tauri event listeners via ESM dynamic import
  useEffect(() => {
    let cleanup1: (() => void) | null = null;
    let cleanup2: (() => void) | null = null;
    let cancelled = false;

    (async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        if (cancelled) return;

        const unlisten1 = await listen('ai-model-download-progress', (event: any) => {
          eventsReceived.current = true;
          const p = event.payload;
          setProgress(p.percent || 0);
          setDownloadedMB(Math.round((p.downloaded || 0) / 1024 / 1024));
          setTotalMB(Math.round((p.total || 0) / 1024 / 1024));
        });
        if (cancelled) { unlisten1(); return; }
        cleanup1 = unlisten1;

        const unlisten2 = await listen('ai-model-download-complete', (event: any) => {
          eventsReceived.current = true;
          setDownloading(false);
          if (event.payload.success) {
            setProgress(100);
            aiModelApi.status().then(setStatus);
            // Server auto-starts after install — check health after a moment
            setTimeout(() => aiModelApi.health().then(setServerRunning), 3000);
          } else {
            setError(event.payload.error || 'Download failed');
          }
        });
        if (cancelled) { unlisten2(); return; }
        cleanup2 = unlisten2;
      } catch {
        // Not in Tauri environment — polling fallback will handle it
      }
    })();

    return () => {
      cancelled = true;
      cleanup1?.();
      cleanup2?.();
    };
  }, []);

  // Fallback: poll model status every 3s while downloading, in case events don't arrive
  useEffect(() => {
    if (!downloading) return;

    const interval = setInterval(async () => {
      // If we're already getting real events, don't poll
      if (eventsReceived.current) return;

      const s = await aiModelApi.status();
      if (s.installed) {
        setDownloading(false);
        setProgress(100);
        setStatus(s);
        return;
      }
      // Check if model file is partially downloaded by checking size
      if (s.modelSizeBytes && s.modelSizeBytes > 0) {
        const pct = Math.min(99, Math.round((s.modelSizeBytes / 1_000_000_000) * 100));
        setProgress(pct);
        setDownloadedMB(Math.round(s.modelSizeBytes / 1024 / 1024));
        setTotalMB(1000); // ~1GB estimate
      }
    }, 3000);

    return () => clearInterval(interval);
  }, [downloading]);

  async function handleDownload() {
    setDownloading(true);
    setProgress(0);
    setError(null);
    setDownloadedMB(0);
    setTotalMB(0);
    eventsReceived.current = false;
    try {
      await aiModelApi.download();
    } catch (e) {
      setError(String(e));
      setDownloading(false);
    }
  }

  async function handleStart() {
    setStarting(true);
    try {
      await aiModelApi.start();
      // Poll for health
      setTimeout(async () => {
        const healthy = await aiModelApi.health();
        setServerRunning(healthy);
        setStarting(false);
      }, 3000);
    } catch (e) {
      setError(String(e));
      setStarting(false);
    }
  }

  async function handleStop() {
    await aiModelApi.stop();
    setServerRunning(false);
  }

  const isInstalled = status?.installed === true;

  return (
    <div>
      <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 8, fontWeight: 600 }}>
        AI Search
      </div>
      <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 12, lineHeight: 1.5 }}>
        Ask questions about your finances in plain English.
        Basic search works instantly. Install the AI model for smarter answers to complex questions.
      </div>

      <div style={{
        padding: 12,
        border: '1px solid var(--border-color)',
        borderRadius: 6,
        background: 'var(--bg-hover)',
        fontSize: 12,
        lineHeight: 1.6,
      }}>
        <div style={{ marginBottom: 8 }}>
          <strong style={{ color: 'var(--text-primary)' }}>AI Search Model</strong>
          <span style={{ color: 'var(--text-muted)', marginLeft: 8 }}>
            {status?.modelName || 'Qwen 2.5 Coder 1.5B'}
          </span>
        </div>

        {error && (
          <div style={{
            color: 'var(--color-danger, #ef4444)',
            marginBottom: 8,
            padding: '6px 8px',
            background: 'rgba(239,68,68,0.1)',
            borderRadius: 4,
          }}>
            {error}
          </div>
        )}

        {!isInstalled ? (
          <>
            <div style={{ color: 'var(--text-muted)', marginBottom: 8 }}>
              Status: <span style={{ color: 'var(--color-danger, #ef4444)' }}>Not installed</span>
              <br />Size: ~1 GB download
              <br />Requires: 2 GB+ RAM
              <br />Runs: 100% on your device. Your data never leaves your machine.
            </div>
            {downloading ? (
              <div>
                <div style={{
                  height: 8, borderRadius: 4,
                  background: 'var(--bg-secondary)',
                  overflow: 'hidden', marginBottom: 6,
                  border: '1px solid var(--border-color)',
                }}>
                  <div style={{
                    height: '100%', borderRadius: 3,
                    background: 'var(--color-accent)',
                    width: `${Math.max(progress, 1)}%`,
                    transition: 'width 0.5s ease',
                  }} />
                </div>
                <div style={{ color: 'var(--text-primary)', fontWeight: 500 }}>
                  Downloading... {progress}%
                  {totalMB > 0 && (
                    <span style={{ color: 'var(--text-muted)', fontWeight: 400, marginLeft: 8 }}>
                      ({downloadedMB} / {totalMB} MB)
                    </span>
                  )}
                </div>
                <div style={{ color: 'var(--text-muted)', marginTop: 4, fontSize: 11 }}>
                  This may take a few minutes depending on your connection.
                  Do not close the app.
                </div>
              </div>
            ) : (
              <Button size="sm" onClick={handleDownload}>Install AI Search</Button>
            )}
          </>
        ) : (
          <>
            <div style={{ color: 'var(--text-muted)', marginBottom: 8 }}>
              Status: <span style={{ color: 'var(--color-accent)' }}>Installed</span>
              {status?.modelSizeBytes && (
                <> ({(status.modelSizeBytes / 1024 / 1024).toFixed(0)} MB)</>
              )}
              <br />
              Server: <span style={{ color: serverRunning ? 'var(--color-accent)' : 'var(--text-muted)' }}>
                {serverRunning ? 'Running' : 'Stopped'}
              </span>
            </div>
            <div style={{ display: 'flex', gap: 8 }}>
              {!serverRunning ? (
                <Button size="sm" onClick={handleStart} disabled={starting}>
                  {starting ? 'Starting...' : 'Start AI Search'}
                </Button>
              ) : (
                <Button size="sm" variant="ghost" onClick={handleStop}>Stop</Button>
              )}
            </div>
          </>
        )}
      </div>

      <div style={{ fontSize: 11, color: 'var(--text-muted)', marginTop: 8 }}>
        Privacy: All search processing runs locally. Your data never leaves your device.
      </div>
    </div>
  );
}
