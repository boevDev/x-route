import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { dnsApi } from '../api/dns';
import type { DnsTarget } from '../types/dns';

export const dnsStatusQueryKey = ['dns-status'] as const;

export function useDnsStatus() {
  return useQuery({
    queryKey: dnsStatusQueryKey,
    queryFn: dnsApi.getStatus,
    refetchInterval: 5000, // подстрахуемся, если DNS поменяли в обход приложения
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
      // Rust-команда уже вернула перечитанное из ОС состояние — используем его напрямую,
      // без лишнего запроса и без setTimeout "подождать, пока применится".
      queryClient.setQueryData(dnsStatusQueryKey, freshStatus);
    },
    onSettled: () => {
      // На всякий случай — если onSuccess не сработал (напр. частичная ошибка) —
      // всё равно синхронизируемся с реальностью.
      queryClient.invalidateQueries({ queryKey: dnsStatusQueryKey });
    },
  });
}
