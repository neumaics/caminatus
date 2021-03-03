import React, { useState, useEffect } from 'react';
import styled from 'styled-components';

const DEFAULT_BUILD = {
  arch: 'unknown',
  branch: 'unknown',
  commit: 'unknown',
  date: 'unknown',
  version: 'unknown',
};

const buildInfo = () => fetch(`http://${location.host}/build-info`).then(r => r.json());

export const Settings = () => {
  const [build, setBuild] = useState(DEFAULT_BUILD);
  
  useEffect(() => {
    buildInfo().then((b) => setBuild(b));
  }, []);
  
  return (
    <div>
      <ul>
        {Object.entries(build).map((kv, i) => <li key={i}>{kv[0]}: {kv[1]}</li>)}
      </ul>
    </div>
  );
};
