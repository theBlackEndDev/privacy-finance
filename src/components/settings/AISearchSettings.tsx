import { useState, useEffect } from 'react';
import { aiModelApi, ModelStatus } from '../../api/ai_model';
import { Button } from '../ui/Button';
import { listen } from '@tauri-apps/api/event';

export function AISearchSettings() {
  const [status, setStatus] = useState<ModelStatus | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [serverRunning, setServerRunning] = useState(false);
  const [starting, setStopping] = useState(false);

  useEffect(() => {
    aiModelApi.status().then(setStatus);
    aiModelApi.health().then(setServerRunning);
  }, []);

  useEffect(() => {
    const unlisten1 = listen('ai-model-download-progress', (event: any) => {
      setProgress(event.payload.percent || 0);
    });
    const unlisten2 = listen('ai-model-download-complete', (event: any) => {
      setDownloading(false);
      if (event.payload.success) {
        aiModelApi.status().then(setStatus);
      }
    });
    return () => { unlisten1.then(f => f()); unlisten2.then(f => f()); };
  }, []);

  async function handleDownload() {
    setDownloading(true);
    setProgress(0);
    await aiModelApi.download();
  }

  async function handleStart() {
    setStopping(true);
    await aiModelApi.start();
    // Wait a moment then check health
    setTimeout(async () => {
      const healthy = await aiModelApi.health();
      setServerRunning(healthy);
      setStopping(false);
    }, 3000);
  }

  async function handleStop() {
    await aiModelApi.stop();
    setServerRunning(false);
  }

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

        {!status?.installed ? (
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
                  height: 6, borderRadius: 3,
                  background: 'var(--bg-secondary)',
                  overflow: 'hidden', marginBottom: 6,
                }}>
                  <div style={{
                    height: '100%', borderRadius: 3,
                    background: 'var(--color-accent)',
                    width: `${progress}%`,
                    transition: 'width 0.3s',
                  }} />
                </div>
                <div style={{ color: 'var(--text-muted)' }}>Downloading... {progress}%</div>
              </div>
            ) : (
              <Button size="sm" onClick={handleDownload}>Install AI Search</Button>
            )}
          </>
        ) : (
          <>
            <div style={{ color: 'var(--text-muted)', marginBottom: 8 }}>
              Status: <span style={{ color: 'var(--color-accent)' }}>Installed</span>
              {status.modelSizeBytes && (
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
