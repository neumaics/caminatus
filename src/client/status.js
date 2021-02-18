import React, { useState, useEffect } from 'react';
import PropTypes from 'prop-types';
import styled from 'styled-components';

const Readout = styled.a`
  font-family: 'IBM Plex Mono', sans-serif;
`;

const subscribe = (id, channel) => 
  fetch(`http://${location.host}/subscribe/${id}/${channel}`, { method: 'post' });

export const StatusBar = (props) => {
  const [temp, setTemp] = useState(0.0.toFixed(2));
  const [setPoint, setSetPoint] = useState(0.0.toFixed(2));
  const [state, setState] = useState('Idle');
  const [id, setId] = useState('');
  const [connected, setConnected] = useState(EventSource.CONNECTING);

  useEffect(() => {
    const eventSource = new EventSource(`http://${location.host}/connect`);
    eventSource.addEventListener('id', (msg) => {
      const clientId = msg.data;
      setId(clientId);

      subscribe(clientId, 'kiln');
      subscribe(clientId, 'system');
    });

    eventSource.onopen = (ev) => {
      console.log(ev);
      setConnected(eventSource.readyState);
    };

    eventSource.onerror = (ev) => {
      console.error("error connecting to server", ev);
    };

    eventSource.addEventListener('kiln', message => {
      const data = JSON.parse(message.data);

      setState(data.state);
      setTemp(data.temperature.toFixed(2));
      setSetPoint(data.setPoint.toFixed(2));
    });

    eventSource.addEventListener('system', message => {
      console.log(message.data);
    });

  }, []);

  return (
    <>
      {/* {clientId} */}
      <a>temperature</a>
      <Readout>{temp}°C</Readout>
      <a>set point</a>
      <Readout>{setPoint}°C</Readout>
      <Readout>{state}</Readout>
    </>
  );
};

StatusBar.propTypes = {
  children: PropTypes.node,
  href: PropTypes.string,
};
