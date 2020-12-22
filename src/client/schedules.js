import React, { useEffect, useState } from 'react';
import styled from 'styled-components';
import { Link } from 'wouter';

const ScheduleGrid = styled.div`
  display: grid;
  grid-auto-rows: 3em;
  padding: .5em
`;

const ScheduleItem = styled.div`
  height: 100%;
  
  &:nth-child(odd) {
    background-color: #C4C4C4;
  }

  &:nth-child(even) {
    background-color: #E3E3E3;
  }
`;
export const Schedules = () => {
  const [schedules, setSchedules] = useState([]);

  const getSchedules = () => fetch('http://localhost:8080/schedules')
    .then(response => response.json())
    .then(setSchedules);
  
  useEffect(getSchedules, []);
  
  const scheduleItems = schedules.map(s => (<ScheduleItem key={s}>
    <Link href={`/app/schedules/${s}`}>{s}</Link>
  </ScheduleItem>));

  return (
  <ScheduleGrid>
    {scheduleItems}
  </ScheduleGrid>);
};

export const Schedule = ({ params }) => {
  const [schedule, setSchedule] = useState({ name: '' });

  const getScheduleInfo = () => fetch(`http://localhost:8080/schedules/${params.scheduleName}`)
    .then(response => response.json())
    .then(setSchedule);

  useEffect(getScheduleInfo, []);
  return (<div>
    <span>{schedule.name}</span>
  </div>);
};

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