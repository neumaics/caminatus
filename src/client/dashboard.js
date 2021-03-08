import React, { useContext, useEffect, useState } from 'react';
import * as Scale from '@visx/scale';
import styled from 'styled-components';
import {
  AnimatedAxis,
  AnimatedGrid,
  LineSeries,
  XYChart,
  Tooltip,
} from '@visx/xychart';
import { ParentSize } from '@visx/responsive';

import { byName, toGraphN } from './schedule/common';
import { ServerEventsContext } from './server-events';

const GraphContainer = styled.div`
  height: 100%;
  width: 100%;
  background: #0A0F12;
  border-radius: 5px;

  .axis-label {
    fill: white;
    font-family: "Work Sans", sans-serif;
  }

  .axis-tick {
    > * g {
      fill: white;
      font-family: "Work Sans", sans-serif;

    }
  }
`;

const accessors = {
  xAccessor: d => d.x,
  yAccessor: d => d.y,
};

export const Dashboard = () => {
  const [schedule, setSchedule] = useState({ name: '' });
  const [liveTemp, setLiveTemp] = useState([]);
  const [setTemp, setSetTemp] = useState([]);
  const [graphData, setGraphData] = useState([]);

  useEffect(() => {
    byName('fast', true).then((s) => {
      setGraphData(toGraphN(s));
      setSchedule(s);
    });
  }, []);
  
  const c = useContext(ServerEventsContext);
  useEffect(() => {
    const unregister = c && typeof c.register === 'function' && c.register('kiln', (message) => {
      const data = JSON.parse(message.data);

      if (data.state.toLowerCase() === 'running') {
        const measureddatapoint = { x: data.runtime, y: data.temperature };
        const setdatapoint = { x: data.runtime, y: data.setPoint };

        liveTemp.push(measureddatapoint);
        setTemp.push(setdatapoint);

        setLiveTemp(liveTemp);
        setSetTemp(setTemp);
      }

      if (data.state.toLowerCase() === 'idle') {
        setLiveTemp([]);
        setSetTemp([]);
      }
    });

    return () => {
      unregister && unregister();
    };
  }, [c]);

  return (
    <GraphContainer>
      {/* <span>{schedule.name}</span> */}
      <ParentSize>
        {parent => <XYChart
          height={parent.height}
          width={parent.width}
          xScale={{ type: 'linear' }}
          yScale={{ type: 'linear' }}>
          <AnimatedAxis
            orientation='bottom'
            label='time (seconds)'
            labelClassName='axis-label'
            tickClassName='axis-tick'
            numTicks={4}
          />
          <AnimatedAxis
            scale={Scale.scaleLinear({
              domain: [0, 2000],
              range: [0, parent.height],
              nice: true
            })}
            label='Temperature (°C)'
            orientation='left'
            labelClassName='axis-label'
            tickClassName='axis-tick'
          />
          <AnimatedGrid columns={false} numTicks={4} />
          <LineSeries dataKey='scheduled' data={graphData} {...accessors} />
          <LineSeries dataKey='live' data={liveTemp} {...accessors} />
          <LineSeries dataKey='set' data={setTemp} {...accessors} />
          <Tooltip
            snapTooltipToDatumX
            snapTooltipToDatumY
            showVerticalCrosshair
            showSeriesGlyphs
            renderTooltip={({ tooltipData, colorScale }) => (
              <div>
                <div style={{ color: colorScale(tooltipData.nearestDatum.key) }}>
                  {tooltipData.nearestDatum.key}
                </div>
                {accessors.xAccessor(tooltipData.nearestDatum.datum)}
                {', '}
                {accessors.yAccessor(tooltipData.nearestDatum.datum)}
              </div>
            )}
          />

        </XYChart>}
      </ParentSize>
    </GraphContainer>
  );
};
