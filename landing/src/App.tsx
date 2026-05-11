import { Nav } from './components/Nav';
import { Hero } from './components/Hero';
import { BotsSection } from './components/BotsSection';
import { EngineSection } from './components/EngineSection';
import { SafetySection } from './components/SafetySection';
import { CtaSection } from './components/CtaSection';
import { Footer } from './components/Footer';

export default function App() {
  return (
    <div className="min-h-screen">
      <Nav />
      <main>
        <Hero />
        <BotsSection />
        <EngineSection />
        <SafetySection />
        <CtaSection />
      </main>
      <Footer />
    </div>
  );
}
