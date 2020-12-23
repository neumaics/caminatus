import React, { useEffect, useState } from 'react';
import styled from 'styled-components';
import { VictoryChart, VictoryTheme, VictoryLine } from 'victory';
import { Link } from 'wouter';

export const TEMPERATURE_SCALE = {
  CELSIUS: 'Celsius',
  FAHRENHEIT: 'Fahrenheit',
  KELVIN: 'Kelvin',
};

export const TIME_SCALE = {
  HOURS: 'hours',
  HOUR: 'hour',
  MINUTES: 'minutes',
  MINUTE: 'minute',
  SECONDS: 'seconds',
  SECOND: 'second',
};

export const TIME_SECONDS = {
  [TIME_SCALE.HOURS]: 3600,
  [TIME_SCALE.HOUR]: 3600,
  [TIME_SCALE.MINUTES]: 60,
  [TIME_SCALE.MINUTE]: 60,
  [TIME_SCALE.SECONDS]: 1,
  [TIME_SCALE.SECOND]: 1,
};

const durationToSeconds = (duration) => duration.value * TIME_SECONDS[duration.unit.toLowerCase()];
const rateToSeconds = (step) => {
  const delta = step.end_temperature - step.start_temperature;
  const p = Math.abs(delta) / step.rate.value;
  const time = p * TIME_SECONDS[step.rate.unit.toLowerCase()];

  return time;
};

/**
 * Maps a schedule to a x/y axis datapoint form.
 * 
 * @fixme this is super inefficient.
 * @param {*} schedule 
 */
const toGraph = (schedule) => {
  const { norm } = schedule.steps.reduce((acc, step) =>{
    const endTime = step.duration ? durationToSeconds(step.duration) : rateToSeconds(step);

    if (acc.norm.find((s) => s.time === acc.time) === undefined) {
      acc.norm.push({ time: acc.time, temp: step.start_temperature});
    }
    
    if (acc.norm.find((s) => s.time === endTime) === undefined) {
      acc.norm.push({ time: acc.time + endTime, temp: step.end_temperature });
    }

    return {
      ...acc,
      time: acc.time + endTime
    };
  }, { norm: [], time: 0 });

  return norm;
};

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
  
  const scheduleItems = schedules.map(s => (
  <ScheduleItem key={s}>
    <Link href={`/app/schedules/${s}`}>{s}</Link>
  </ScheduleItem>));

  return (
  <div>
    <button><Link href='/app/schedules/create'>+</Link></button>
    <ScheduleGrid>{scheduleItems}</ScheduleGrid>
  </div>
  );
};

export const Schedule = ({ params }) => {
  const [schedule, setSchedule] = useState(null);

  const getScheduleInfo = () => fetch(`http://localhost:8080/schedules/${params.scheduleName}`)
    .then(response => response.json())
    .then(setSchedule);

  const graphData = schedule && toGraph(schedule);

  const graph = (
    <VictoryChart
      theme={VictoryTheme.material}
      padding={{ top: 0, bottom: 0, left: 40, right: 40 }}
      height={75}
    >
      <VictoryLine 
        style={{ data: { stroke: '#C43A31' }, parent: { border: '1px solid #CCC' }}}
        data={graphData}
        x='time'
        y='temp'
      />
    </VictoryChart>
  )
  useEffect(getScheduleInfo, []);
  return (schedule && <div>
    <div>{schedule.name}</div>
    {graph}
    <div>{schedule.description}</div>
    <div>{schedule.scale}</div>
    <div>
      {schedule.steps.map((s, i) => (<div key={i}>
        <div>{s.start_temperature}</div>
        <div>{s.end_temperature}</div>
      </div>))}
    </div>
  </div>);
};

const Form = styled.form`
`;

export const CreateSchedule = () => {
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