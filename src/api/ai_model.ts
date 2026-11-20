import { invoke } from './invoke';

export interface ModelStatus {
  installed: boolean;
  modelName: string;
  modelPath: string | null;
  modelSizeBytes: number | null;
  ready: boolean;
}

export const aiModelApi = {
  status: () => invoke<ModelStatus>('ai_model_status', {}, {
    installed: false, modelName: '', modelPath: null, modelSizeBytes: null, ready: false,
  }),
  download: () => invoke<void>('ai_model_download'),
};
