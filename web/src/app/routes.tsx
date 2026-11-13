import { createHashRouter } from 'react-router';
import { Play } from '../pages/Play';
import { Layout } from './Layout';

export const router = createHashRouter([
  {
    element: <Layout />,
    children: [
      { index: true, element: <Play /> },
      { path: 'outbreak', lazy: async () => ({ Component: (await import('../pages/Outbreak')).Outbreak }) },
      { path: 'activity', lazy: async () => ({ Component: (await import('../pages/Activity')).Activity }) },
      { path: 'how-to-play', lazy: async () => ({ Component: (await import('../pages/HowToPlay')).HowToPlay }) },
      { path: 'faq', lazy: async () => ({ Component: (await import('../pages/Faq')).FaqPage }) },
    ],
  },
  { path: 'brand', lazy: async () => ({ Component: (await import('../pages/Brand')).Brand }) },
  { path: 'brand/og', lazy: async () => ({ Component: (await import('../pages/Brand')).BrandOg }) },
  { path: 'brand/icon', lazy: async () => ({ Component: (await import('../pages/Brand')).BrandIcon }) },
]);
