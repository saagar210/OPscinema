import { ipc } from '../../ipc/typed';

export interface PermissionsViewModel {
  screen_recording: boolean;
  accessibility: boolean;
  full_disk_access: boolean;
  offline_mode: boolean;
}

export async function loadPermissionsView(): Promise<PermissionsViewModel> {
  const permissions = await ipc.app_get_permissions_status({});

  const settings = await ipc.settings_get({});

  return {
    screen_recording: permissions.ok ? permissions.value.screen_recording : false,
    accessibility: permissions.ok ? permissions.value.accessibility : false,
    full_disk_access: permissions.ok ? permissions.value.full_disk_access : false,
    offline_mode: settings.ok ? settings.value.offline_mode : true,
  };
}
