import React, { useEffect, useState } from 'react';

export const Schedules = (_props) => {
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
  return <span>{schedule.name}</span>;
};
