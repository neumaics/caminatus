import React, { useEffect, useState } from 'react';
import styled from 'styled-components';
export const Schedules = () => {
  const [schedules, setSchedules] = useState([]);

  const getSchedules = () => fetch('http://localhost:8080/schedules')
    .then(response => response.json())
    .then(setSchedules);
  
  useEffect(getSchedules, []);

  const scheduleItems = schedules.map(s => <Schedule key={s} scheduleId={s} />);

  return <div>{scheduleItems}</div>
};

export const Schedule = ({ scheduleId }) => {
  const [schedule, setSchedule] = useState({ name: '' });

  const getScheduleInfo = () => fetch(`http://localhost:8080/schedules/${scheduleId}`)
    .then(response => response.json())
    .then(setSchedule);

  useEffect(getScheduleInfo, []);
  return (<div>
    <span>{schedule.name}</span>
    {/* <NewScheduleForm /> */}
  </div>);
};

Schedule.propTypes = {
  scheduleId: '',
}

export const TEMPERATURE_SCALE = {
  CELSIUS: 'Celsius',
  FAHRENHEIT: 'Fahrenheit',
  KELVIN: 'Kelvin',
};

export const TIME_SCALE = {
  HOURS: 'hours',
  MINUTES: 'minutes',
  SECONDS: 'seconds',
}

const Form = styled.form`
`;

export const NewScheduleForm = () => {
  // const step = {
  //   description: '',
  //   startTemperature: 0.0,
  //   endTemperature: 0.0,
  //   duration: null,
  //   rate: null,
  // };

  // const schedule = {
  //   name: '',
  //   description: '',
  //   scale: TEMPERATURE_SCALE.CELCIUS,
  //   steps: []
  // };
  
  return (
    <Form>
      <input placeholder="name" />
      <input placeholder="description" />
      <select>
        {Object.values(TEMPERATURE_SCALE).map(s => <option key={s} value={s}>{s}</option>)}
      </select>
      <div>
        <button>-</button>
        <select>
          <option>rate</option>
          <option>duration</option>
        </select>
        <input type="number" placeholder="start" />
        <input type="number" placeholder="end" />
        <input placeholder="0.0" />
        <select>
          {Object.values(TIME_SCALE).map(t => <option key={t} value={t}>{t}</option>)}
        </select>
      </div>
      <button>+</button>
    </Form>
  );
};