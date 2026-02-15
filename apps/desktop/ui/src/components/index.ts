export interface InlineNotice {
  level: 'info' | 'warning' | 'error';
  message: string;
}

export function toInlineNoticeFromErrorCode(code: string): InlineNotice {
  if (code === 'PERMISSION_DENIED') {
    return {
      level: 'warning',
      message: 'Permissions are required before capture can start.',
    };
  }
  if (code === 'EXPORT_GATE_FAILED') {
    return {
      level: 'error',
      message: 'Export was blocked by strict evidence or anchor policy checks.',
    };
  }
  return {
    level: 'info',
    message: 'Action completed with a recoverable issue.',
  };
}
