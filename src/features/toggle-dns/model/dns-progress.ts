import type { DnsProgressStage, DnsToggleOperation } from '@/entities/dns';

export type DnsWorkingStage = Exclude<DnsProgressStage, 'done' | 'error'>;

export type DnsToggleState =
  | { kind: 'idle' }
  | { kind: 'working'; operation: DnsToggleOperation; stage: DnsWorkingStage }
  | { kind: 'success'; operation: DnsToggleOperation }
  | { kind: 'error'; message: string };

export type DnsToggleAction =
  | { type: 'start'; operation: DnsToggleOperation }
  | { type: 'stage'; stage: DnsWorkingStage }
  | { type: 'finish' }
  | { type: 'fail'; message: string }
  | { type: 'dismiss' };

export const initialDnsToggleState: DnsToggleState = { kind: 'idle' };

export function dnsToggleReducer(state: DnsToggleState, action: DnsToggleAction): DnsToggleState {
  switch (action.type) {
    case 'start':
      return { kind: 'working', operation: action.operation, stage: 'resolving' };
    case 'stage':
      return state.kind === 'working' ? { ...state, stage: action.stage } : state;
    case 'finish':
      return state.kind === 'working' ? { kind: 'success', operation: state.operation } : state;
    case 'fail':
      return state.kind === 'working' ? { kind: 'error', message: action.message } : state;
    case 'dismiss':
      return { kind: 'idle' };
    default:
      return state;
  }
}

export function workingStageLabel(operation: DnsToggleOperation, stage: DnsWorkingStage): string {
  if (operation === 'enable') {
    switch (stage) {
      case 'resolving':
        return 'Определяем сетевой адаптер…';
      case 'applying':
        return 'Подключаем DNS-сервера…';
      case 'clearing':
        return 'Обновляем кэш DNS…';
      case 'verifying':
        return 'Проверяем применённые настройки…';
    }
  }

  switch (stage) {
    case 'resolving':
      return 'Определяем сетевой адаптер…';
    case 'resetting':
      return 'Возвращаем автоматический DNS…';
    case 'clearing':
      return 'Очищаем кэш распознавания имён…';
    case 'verifying':
      return 'Проверяем применённые настройки…';
    default:
      return 'Применяем настройки…';
  }
}

export function successLabel(operation: DnsToggleOperation): string {
  return operation === 'enable' ? 'DNS подключён' : 'Автоматический DNS восстановлен';
}
