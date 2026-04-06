import { BrowserRouter, Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";
import Dashboard from "./pages/Dashboard";
import Rules from "./pages/Rules";
import DnsServers from "./pages/DnsServers";
import QueryLog from "./pages/QueryLog";
import Settings from "./pages/Settings";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="rules" element={<Rules />} />
          <Route path="dns-servers" element={<DnsServers />} />
          <Route path="query-log" element={<QueryLog />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
