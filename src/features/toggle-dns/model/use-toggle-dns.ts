import { useEffect, useMemo, useReducer } from 'react';
import { isDnsTargetActive, useDnsPreferences, useDnsStatus, useSetDns } from '@/entities/dns';
import { getErrorMessage } from '@/shared/lib/error';
import { dnsToggleReducer, initialDnsToggleState } from './dns-progress';
import { useDnsProgressEvents, useDnsSuccessTimeout } from './use-dns-progress';

export function useToggleDns() {
  const target = useDnsPreferences((state) => state.target);
  const { data: status, isLoading, error: statusError } = useDnsStatus();
  const setDnsMutation = useSetDns();

  const [toggle, dispatch] = useReducer(dnsToggleReducer, initialDnsToggleState);
  useDnsProgressEvents(dispatch);
  useDnsSuccessTimeout(toggle, dispatch);

  useEffect(() => {
    if (setDnsMutation.isSuccess) dispatch({ type: 'finish' });
  }, [setDnsMutation.isSuccess, dispatch]);

  useEffect(() => {
    if (!setDnsMutation.isError) return;
    const message = getErrorMessage(setDnsMutation.error);
    if (message !== null) dispatch({ type: 'fail', message });
  }, [setDnsMutation.isError, setDnsMutation.error, dispatch]);

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
      : (getErrorMessage(setDnsMutation.error) ?? getErrorMessage(statusError));

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
