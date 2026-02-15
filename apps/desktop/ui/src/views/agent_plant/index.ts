import { ipc } from '../../ipc/typed';

export interface AgentPlantViewModel {
  pipelines: string[];
  can_run: boolean;
}

export async function loadAgentPlantView(sessionId: string | undefined): Promise<AgentPlantViewModel> {
  const pipelines = await ipc.agent_pipelines_list({});
  return {
    pipelines: pipelines.ok ? pipelines.value.pipelines : [],
    can_run: Boolean(sessionId),
  };
}
