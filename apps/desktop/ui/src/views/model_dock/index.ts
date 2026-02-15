import { ipc } from '../../ipc/typed';

export interface ModelDockViewModel {
  model_count: number;
  bench_count: number;
  role_map: Record<string, string>;
}

export async function loadModelDockView(): Promise<ModelDockViewModel> {
  const models = await ipc.models_list({
    include_unhealthy: false,
  });

  const benches = await ipc.bench_list({});

  const roles = await ipc.model_roles_get({});

  return {
    model_count: models.ok ? models.value.models.length : 0,
    bench_count: benches.ok ? benches.value.benches.length : 0,
    role_map: roles.ok
      ? {
          tutorial_generation: roles.value.tutorial_generation ?? '',
          screen_explainer: roles.value.screen_explainer ?? '',
          anchor_grounding: roles.value.anchor_grounding ?? '',
        }
      : {},
  };
}
