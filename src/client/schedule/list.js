import React, { useEffect, useState } from 'react';
import styled from 'styled-components';
import { Link } from 'wouter';

import { StatusBar } from '../status';

const ScheduleGrid = styled.div`
  display: grid;
  grid-auto-rows: 3em;
  border-radius: 0.5em;
  overflow: hidden;
`;

const ScheduleItem = styled.div`
  height: 100%;

  &:nth-child(even) {
    background-color: #E3E3E3;
  }

  &:nth-child(odd) {
    background-color: #C4C4C4;
  }
`;

const ScheduleMenu = styled.div`
  margin-bottom: 1em;
`;

export const Schedules = () => {
  const [schedules, setSchedules] = useState([]);

  const getSchedules = () => fetch(`http://${location.host}/schedules`)
    .then(response => response.json())
    .then(setSchedules);
  
  useEffect(getSchedules, []);
  
  const scheduleItems = schedules.map(s => (
    <ScheduleItem key={s}>
      <Link href={`/app/schedules/${s}`}>{s}</Link>
    </ScheduleItem>));

  for (let i = 6; scheduleItems.length < 6; i--) {
    scheduleItems.push(<ScheduleItem key={i}></ScheduleItem>);
  }

  return (
    <>
      <StatusBar></StatusBar>
      <ScheduleMenu>
        <button><Link href='/app/schedules/create'>+</Link></button>
      </ScheduleMenu>
      <ScheduleGrid>
        {scheduleItems}
      </ScheduleGrid>
    </>
  );
};
