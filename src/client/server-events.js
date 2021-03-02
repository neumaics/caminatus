import React, { createContext, useState, useEffect } from 'react';
import PropTypes from 'prop-types';

const DEFAULT_STATE = { register: () => {} }; //noop
export const ServerEventsContext = createContext(DEFAULT_STATE);

const subscribe = (id, channel) => 
  fetch(`http://${location.host}/subscribe/${id}/${channel}`, { method: 'post' });

export const SeverEventsProvider = (props) => {
  const [eventSource, setEventSource] = useState(DEFAULT_STATE);
  
  function register(id, evtsource) {
    return {
      register: (channel, callback) => {
        subscribe(id, channel)
          .catch(error => console.error(`error regsitering`, error));

        evtsource.addEventListener(channel, callback);

        return () => evtsource.removeEventListener(channel, callback);
      },
    };
  }

  useEffect(() => {
    const source = new EventSource(`http://${location.host}/connect`);

    source.addEventListener('id', (msg) => {
      const clientId = msg.data;

      subscribe(clientId, 'system');
      setEventSource(register(clientId, source));
    });

    source.onerror = (ev) => {
      console.error('error connecting to server', ev);
    };
  }, []);

  return (
    <ServerEventsContext.Provider value={eventSource}>
      {props.children}
    </ServerEventsContext.Provider>
  );
};

SeverEventsProvider.propTypes = {
  children: PropTypes.arrayOf(PropTypes.node)
};
