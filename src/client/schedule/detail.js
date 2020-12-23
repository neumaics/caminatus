import React, { useEffect, useState } from 'react';
import styled from 'styled-components';

const ScheduleDetail = styled.div`

`;

const ScheduleData = styled.div`
  background-color: #C4C4C4;
  border-radius: .5em;
  color: #2C3033;
  padding: 1em;
  margin-top: 1em;
`;

const StepTable = styled.table`
  width: 100%;
  max-width: 600px;
  padding-top: 1em;

  tbody tr:nth-child(odd) {
    background-color: #F1F1F1;
  }

  tbody tr:nth-child(even) {
    background-color: #E3E3E3;
  }
`;

export const Schedule = ({ params }) => {
  const [schedule, setSchedule] = useState(null);

  const getScheduleInfo = () => fetch(`http://localhost:8080/schedules/${params.scheduleName}`)
    .then(response => response.json())
    .then(setSchedule);

  // const graphData = schedule && toGraph(schedule);

  const graph = (
    <div></div>
  );

  useEffect(getScheduleInfo, []);
  return (schedule && <ScheduleDetail>
    <span>{schedule.name}</span>
    {graph}
    <ScheduleData>
      <div>{schedule.description}</div>
      {/* <div>{schedule.scale}</div> */}
      <StepTable>
        <thead>
          <tr>
            <th></th>
            <th>start</th>
            <th>end</th>
            <th>increment</th>
            <th>time</th>
          </tr>
        </thead>
        <tbody>
          {schedule.steps.map((s, i) => (<tr key={i}>
            <td>{i + 1}</td>
            <td>{s.start_temperature}</td>
            <td>{s.end_temperature}</td>
            <td>{s.rate ? s.rate.value : s.duration.value}</td>
            <td>{s.rate ? s.rate.unit : s.duration.unit}</td>
          </tr>))}
        </tbody>
      </StepTable>
    </ScheduleData>
  </ScheduleDetail>);
};
