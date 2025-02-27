/**
 * Copyright 2020 Alibaba Group Holding Limited.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package com.alibaba.maxgraph.tinkerpop.steps;

import org.apache.tinkerpop.gremlin.process.traversal.Traversal;
import org.apache.tinkerpop.gremlin.process.traversal.Traverser;
import org.apache.tinkerpop.gremlin.process.traversal.step.map.FlatMapStep;
import org.apache.tinkerpop.gremlin.structure.Element;
import org.apache.tinkerpop.gremlin.structure.Vertex;

import java.util.Iterator;

public class PageRankStep<E extends Element> extends FlatMapStep<Vertex, E> {
    public final int iteration;
    public final String outPropId;
    public final double alpha;

    public PageRankStep(
            Traversal.Admin traversal, String outPropId, final double alpha, int iteration) {
        super(traversal);
        this.outPropId = outPropId;
        this.iteration = iteration;
        this.alpha = alpha;
    }

    @Override
    protected Iterator<E> flatMap(Traverser.Admin<Vertex> traverser) {
        throw new IllegalArgumentException();
    }
}
