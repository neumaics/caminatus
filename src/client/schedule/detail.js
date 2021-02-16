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

  const getScheduleInfo = () => fetch(`http://${location.host}/schedules/${params.scheduleName}`)
    .then(response => response.json())
    .then(setSchedule);

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
        </thead>
        <tbody>
          {schedule.steps.map((s, i) => (<tr key={i}>
            <td>{i + 1}</td>
            <td>{s}</td>

          </tr>))}
        </tbody>
      </StepTable>
    </ScheduleData>
  </ScheduleDetail>);
};
