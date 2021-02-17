import React, { useState, useEffect } from 'react';
import PropTypes from 'prop-types';
import styled from 'styled-components';

export const StatusBar = (props) => {
  const [temp, setTemp] = useState(0.0);
  const [setPoint, setSetPoint] = useState(0.0);
  const [state, setState] = useState('Idle');
  const [id, setId] = useState('');

  useEffect(() => {
    const eventSource = new EventSource(`http://${location.host}/connect`);
    eventSource.addEventListener('id', (msg) => {
      const clientId = msg.data;
      console.log(clientId);
      setId(clientId);

      fetch(`http://${location.host}/subscribe/${clientId}/kiln`, { method: 'post' });
      fetch(`http://${location.host}/subscribe/${clientId}/system`, { method: 'post' });
    });

    eventSource.addEventListener('kiln', message => {
      const data = JSON.parse(message.data);

      setState(data.state);
      setTemp(data.temperature);
      setSetPoint(data.setPoint);
    });

    
    eventSource.addEventListener('system', message => {
      console.log(message.data);
    });

  }, []);

  return (
    <>
      <p>{temp}</p>
      <p>{setPoint}</p>
      <p>{state}</p>
    </>
  );
};

StatusBar.propTypes = {
  children: PropTypes.node,
  href: PropTypes.string,
};
