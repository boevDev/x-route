import { MainPage } from '@/pages/main';
import { QueryProvider } from './providers/QueryProvider';
import './styles/global.css';

export function App() {
  return (
    <QueryProvider>
      <MainPage />
    </QueryProvider>
  );
}
