import { invoke } from './invoke';
import { Agent, CreateAgentInput, UpdateAgentInput, AgentKey, AgentKeyCreated, ScopeGrant, AuditEntry, ParsedManifest, CuratedAgent } from '../types';

export const agentsApi = {
  getAll: () => invoke<Agent[]>('get_agents', {}, []),
  getOne: (id: string) => invoke<Agent>('get_agent', { id }),
  create: (input: CreateAgentInput) => invoke<Agent>('create_agent', { input }),
  update: (id: string, input: UpdateAgentInput) => invoke<Agent>('update_agent', { id, input }),
  updateStatus: (id: string, status: string) => invoke<Agent>('update_agent_status', { id, status }),
  delete: (id: string) => invoke<void>('delete_agent', { id }),

  // Scopes
  getScopes: (agentId: string) => invoke<ScopeGrant[]>('get_agent_scopes', { agentId }, []),
  setScope: (agentId: string, scope: string, accessMode: string) =>
    invoke<ScopeGrant>('set_agent_scope', { agentId, scope, accessMode }),

  // Keys
  generateKey: (agentId: string, scopes: string[], expiresInDays?: number) =>
    invoke<AgentKeyCreated>('generate_agent_key', { agentId, scopes, expiresInDays }),
  getKeys: (agentId: string) => invoke<AgentKey[]>('get_agent_keys', { agentId }, []),
  revokeKey: (keyId: string) => invoke<void>('revoke_agent_key', { keyId }),

  // Manifest
  parseManifest: (manifestJson: string) =>
    invoke<ParsedManifest>('parse_agent_manifest', { manifestJson }),
  registerFromManifest: (manifestJson: string) =>
    invoke<Agent>('register_from_manifest', { manifestJson }),

  // Curated Directory
  getCuratedAgents: () => invoke<CuratedAgent[]>('get_curated_agents', {}, []),
  installCurated: (curatedId: string) => invoke<Agent>('install_curated_agent', { curatedId }),

  // Audit
  getAuditLog: (agentId?: string, limit?: number) =>
    invoke<AuditEntry[]>('get_audit_log', { agentId, limit }, []),
};
