import { useEffect } from 'react';
import { onDnsProgress } from '../api/dns-progress.api';
import type { DnsToggleAction, DnsToggleState } from './dns-progress';

const FALLBACK_ERROR_MESSAGE = 'Не удалось изменить настройки DNS';

export function useDnsProgressEvents(dispatch: (action: DnsToggleAction) => void) {
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    onDnsProgress(({ stage, detail }) => {
      if (stage === 'done') {
        dispatch({ type: 'finish' });
        return;
      }
      if (stage === 'error') {
        dispatch({ type: 'fail', message: detail ?? FALLBACK_ERROR_MESSAGE });
        return;
      }
      dispatch({ type: 'stage', stage });
    })
      .then((stop) => {
        if (disposed) stop();
        else unlisten = stop;
      })
      .catch(() => {});

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [dispatch]);
}

export function useDnsSuccessTimeout(
  state: DnsToggleState,
  dispatch: (action: DnsToggleAction) => void,
  delayMs = 1800,
) {
  useEffect(() => {
    if (state.kind !== 'success') return undefined;
    const timer = setTimeout(() => dispatch({ type: 'dismiss' }), delayMs);
    return () => clearTimeout(timer);
  }, [state, dispatch, delayMs]);
}
