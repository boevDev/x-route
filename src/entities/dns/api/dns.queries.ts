import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { dnsApi } from './dns.api';
import type { DnsTarget } from '../../../shared/api/dns/dns.types';

export const dnsStatusQueryKey = ['dns-status'] as const;

export function useDnsStatus() {
  return useQuery({
    queryKey: dnsStatusQueryKey,
    queryFn: dnsApi.getStatus,
    staleTime: 30_000,
    refetchOnWindowFocus: true,
    retry: 1,
  });
}

export function useSetDns() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ enable, target }: { enable: boolean; target: DnsTarget }) =>
      dnsApi.setDns(enable, target),
    onSuccess: (freshStatus) => {
      queryClient.setQueryData(dnsStatusQueryKey, freshStatus);
    },
    onError: () => {
      queryClient.invalidateQueries({ queryKey: dnsStatusQueryKey });
    },
  });
}
