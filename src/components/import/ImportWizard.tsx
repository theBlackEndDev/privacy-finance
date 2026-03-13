import { useState, useEffect } from 'react';
import { Account, ColumnMapping, CsvPreview, ImportProfile, ImportResult } from '../../types';
import { importApi } from '../../api/import';
import { FileSelectStep } from './FileSelectStep';
import { PreviewStep } from './PreviewStep';
import { ColumnMappingStep } from './ColumnMappingStep';
import { ImportResultStep } from './ImportResultStep';
import { Spinner } from '../ui/Spinner';

type Step = 'file' | 'preview' | 'mapping' | 'importing' | 'result';

const STEP_LABELS: Record<Step, string> = {
  file: '1. Select File',
  preview: '2. Preview',
  mapping: '3. Map Columns',
  importing: '4. Importing…',
  result: '4. Results',
};

const STEP_ORDER: Step[] = ['file', 'preview', 'mapping', 'importing', 'result'];

interface Props {
  accounts: Account[];
  onNavigateToTransactions: () => void;
}

export function ImportWizard({ accounts, onNavigateToTransactions }: Props) {
  const [step, setStep] = useState<Step>('file');
  const [filePath, setFilePath] = useState('');
  const [accountId, setAccountId] = useState('');
  const [preview, setPreview] = useState<CsvPreview | null>(null);
  const [profiles, setProfiles] = useState<ImportProfile[]>([]);
  const [result, setResult] = useState<ImportResult | null>(null);
  const [importError, setImportError] = useState('');
  const [loadingPreview, setLoadingPreview] = useState(false);

  useEffect(() => {
    importApi.getImportProfiles().then(setProfiles).catch(() => {});
  }, []);

  async function handleFileNext(fp: string, accId: string) {
    setFilePath(fp);
    setAccountId(accId);
    setLoadingPreview(true);
    try {
      const p = await importApi.previewCsv(fp);
      setPreview(p);
      setStep('preview');
    } catch (e) {
      setImportError(`Could not read file: ${e}`);
    } finally {
      setLoadingPreview(false);
    }
  }

  async function handleMappingNext(
    mappings: ColumnMapping[],
    dateFormat: string,
    amountFormat: string,
  ) {
    setStep('importing');
    setImportError('');
    try {
      const r = await importApi.importCsv({
        filePath,
        accountId,
        mappings,
        dateFormat,
        amountFormat,
        profileId: null,
      });
      setResult(r);
      setStep('result');
    } catch (e) {
      setImportError(String(e));
      setStep('mapping');
    }
  }

  function reset() {
    setStep('file');
    setFilePath('');
    setAccountId('');
    setPreview(null);
    setResult(null);
    setImportError('');
  }

  const stepIndex = STEP_ORDER.indexOf(step);
  const displaySteps: Step[] = ['file', 'preview', 'mapping', 'result'];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 24, maxWidth: 720 }}>
      {/* Progress bar */}
      <div style={{ display: 'flex', gap: 0 }}>
        {displaySteps.map((s, i) => {
          const sIdx = STEP_ORDER.indexOf(s);
          const active = s === step || (step === 'importing' && s === 'mapping');
          const done = sIdx < stepIndex && step !== 'importing';
          return (
            <div key={s} style={{ flex: 1, display: 'flex', alignItems: 'center' }}>
              <div style={{
                flex: 1,
                padding: '7px 0',
                textAlign: 'center',
                fontSize: 12,
                fontWeight: active || done ? 600 : 400,
                color: active ? 'var(--color-accent)' : done ? 'var(--color-income)' : 'var(--text-muted)',
                borderBottom: `2px solid ${active ? 'var(--color-accent)' : done ? 'var(--color-income)' : 'var(--border-color)'}`,
              }}>
                {STEP_LABELS[s]}
              </div>
              {i < displaySteps.length - 1 && (
                <div style={{ width: 1, height: 24, background: 'var(--border-color)' }} />
              )}
            </div>
          );
        })}
      </div>

      {/* Step content */}
      <div style={{
        background: 'var(--bg-card)',
        border: '1px solid var(--border-color)',
        borderRadius: 8,
        padding: 24,
      }}>
        {loadingPreview && (
          <div style={{ display: 'flex', justifyContent: 'center', padding: 32 }}>
            <Spinner />
          </div>
        )}

        {!loadingPreview && step === 'file' && (
          <FileSelectStep accounts={accounts} onNext={handleFileNext} />
        )}

        {!loadingPreview && step === 'preview' && preview && (
          <PreviewStep
            preview={preview}
            filePath={filePath}
            onBack={() => setStep('file')}
            onNext={() => setStep('mapping')}
          />
        )}

        {!loadingPreview && step === 'mapping' && preview && (
          <>
            {importError && (
              <div style={{ fontSize: 13, color: 'var(--color-expense)', marginBottom: 12 }}>
                {importError}
              </div>
            )}
            <ColumnMappingStep
              headers={preview.headers}
              profiles={profiles}
              onBack={() => setStep('preview')}
              onNext={handleMappingNext}
              onProfilesChange={setProfiles}
            />
          </>
        )}

        {step === 'importing' && (
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 16, padding: 32 }}>
            <Spinner />
            <div style={{ fontSize: 14, color: 'var(--text-muted)' }}>Importing transactions…</div>
          </div>
        )}

        {step === 'result' && result && (
          <ImportResultStep
            result={result}
            onViewTransactions={onNavigateToTransactions}
            onImportAnother={reset}
          />
        )}
      </div>
    </div>
  );
}
