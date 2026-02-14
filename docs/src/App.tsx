import { HashRouter, Routes, Route } from "react-router-dom";
import { Layout } from "@/components/Layout";
import { Home } from "@/pages/Home";
import { Installation } from "@/pages/Installation";
import { Configuration } from "@/pages/Configuration";
import { Usage } from "@/pages/Usage";
import { Providers } from "@/pages/Providers";
import { Service } from "@/pages/Service";
import { Troubleshooting } from "@/pages/Troubleshooting";
import { Changelog } from "@/pages/Changelog";

export function App() {
  return (
    <HashRouter>
      <Layout>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/installation" element={<Installation />} />
          <Route path="/configuration" element={<Configuration />} />
          <Route path="/usage" element={<Usage />} />
          <Route path="/providers" element={<Providers />} />
          <Route path="/service" element={<Service />} />
          <Route path="/troubleshooting" element={<Troubleshooting />} />
          <Route path="/changelog" element={<Changelog />} />
        </Routes>
      </Layout>
    </HashRouter>
  );
}
