import { useEffect, useMemo, useReducer } from 'react';
import { useDnsStatus, useSetDns } from '../../../entities/dns/api/dns.queries';
import { useDnsPreferences } from '../../../entities/dns/model/dns-preferences.store';
import { isDnsTargetActive } from '../../../shared/lib/dns/dns.util';
import {
  dnsToggleReducer,
  initialDnsToggleState,
  useDnsProgressEvents,
  useDnsSuccessTimeout,
} from './dns-progress';

export function useToggleDns() {
  const target = useDnsPreferences((state) => state.target);
  const { data: status, isLoading, error: statusError } = useDnsStatus();
  const setDnsMutation = useSetDns();

  const [toggle, dispatch] = useReducer(dnsToggleReducer, initialDnsToggleState);
  useDnsProgressEvents(dispatch);
  useDnsSuccessTimeout(toggle, dispatch);

  useEffect(() => {
    if (setDnsMutation.isSuccess) dispatch({ type: 'finish' });
  }, [setDnsMutation.isSuccess]);

  useEffect(() => {
    if (!setDnsMutation.isError || !(setDnsMutation.error instanceof Error)) return;
    dispatch({ type: 'fail', message: setDnsMutation.error.message });
  }, [setDnsMutation.isError, setDnsMutation.error]);

  const isActive = useMemo(() => isDnsTargetActive(status, target), [status, target]);
  const isToggling = toggle.kind === 'working';
  const isBusy = isLoading || isToggling || setDnsMutation.isPending;

  const handleToggle = () => {
    if (isBusy) return;
    const operation = isActive ? 'disable' : 'enable';
    dispatch({ type: 'start', operation });
    setDnsMutation.mutate({ enable: !isActive, target });
  };

  const errorMessage =
    toggle.kind === 'error'
      ? toggle.message
      : setDnsMutation.error instanceof Error
        ? setDnsMutation.error.message
        : statusError instanceof Error
          ? statusError.message
          : null;

  return {
    isActive,
    isBusy,
    toggle,
    status,
    target,
    handleToggle,
    errorMessage,
  };
}
